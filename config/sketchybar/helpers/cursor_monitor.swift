#!/usr/bin/swift

import Foundation
import CoreGraphics
import AppKit

// Configuration
// NOTE: These values are tuned for MacBook Air at 1710x1112 resolution
// May need adjustment for different display resolutions/DPI settings
let topEdgeThreshold: CGFloat = 3.0         // Hide bar when cursor within 3px of top (tight)
let bottomEdgeActiveThreshold: CGFloat = 44.0  // Show bar when cursor moves below 44px from top (loose)
let bouncerThreshold: CGFloat = 5.0         // Prevent cursor from entering top 5px (bouncer)
let bounceTargetOffset: CGFloat = 6.0       // Where to bounce cursor back to (6px from top)
let pollInterval: TimeInterval = 0.1
let sketchybarPath = "/opt/homebrew/bin/sketchybar"

// Display detection state
// When external monitor is primary, bar moves to bottom and auto-hide/bouncer are disabled
var barPosition: String = "top"  // "top" or "bottom"

func getCursorPosition() -> NSPoint {
    return NSEvent.mouseLocation
}

func getScreenTop() -> CGFloat {
    guard let mainScreen = NSScreen.main else {
        return 0
    }
    // In NSScreen coordinates, origin is bottom-left
    // Top of screen is frame.origin.y + frame.size.height
    return mainScreen.frame.origin.y + mainScreen.frame.size.height
}

func isMenuBarMenuOpen() -> Bool {
    // Check if any menu bar menus are currently open
    // Menu windows have specific window levels:
    // - kCGPopUpMenuWindowLevel = 101 (standard menus)
    // - kCGMainMenuWindowLevel = 24 (menu bar itself)

    let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
    guard let windowList = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] else {
        return false
    }

    // Apps that use high window levels but aren't menu bar menus
    // These should be excluded to prevent false positives
    let excludedApps = [
        "QuickTime Player",
        "Screenshot",
        "Screen Recording",
        "screencaptureui"  // macOS screen capture UI
    ]

    for window in windowList {
        // Check window level - menus are at level 101 or higher
        if let level = window[kCGWindowLayer as String] as? Int,
           level >= 101 {
            if let owner = window[kCGWindowOwnerName as String] as? String {
                // Skip if owner is in excluded list (recording controls, etc.)
                if !excludedApps.contains(owner) {
                    // Menu windows are owned by the app that opened them
                    // They have level 101 (kCGPopUpMenuWindowLevel)
                    return true
                }
            }
        }
    }

    return false
}

func triggerSketchyBarEvent(_ eventName: String) {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: sketchybarPath)
    process.arguments = ["--trigger", eventName]

    // Redirect output to /dev/null
    process.standardOutput = FileHandle.nullDevice
    process.standardError = FileHandle.nullDevice

    try? process.run()
}

func isCommandKeyPressed() -> Bool {
    // Check if Command key is currently pressed
    return NSEvent.modifierFlags.contains(.command)
}

func checkAccessibilityPermissions() -> Bool {
    // Check if process has accessibility permissions (required for cursor warping)
    // Setting prompt option to true will show the system dialog if permissions not granted
    let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
    return AXIsProcessTrustedWithOptions(options)
}

func bounceCursor(currentPos: NSPoint, screenTop: CGFloat) {
    // Move cursor back to bounce target position
    // Note: NSEvent.mouseLocation uses Cocoa coordinates (bottom-left origin)
    // but CGWarpMouseCursorPosition expects CG coordinates (top-left origin)

    guard let mainScreen = NSScreen.main else { return }
    let screenHeight = mainScreen.frame.size.height
    let screenOriginY = mainScreen.frame.origin.y

    // Calculate target in Cocoa coordinates (6px from top)
    let targetCocoaY = screenTop - bounceTargetOffset

    // Convert from Cocoa (bottom-left) to CG (top-left)
    // CG Y = total height - Cocoa Y
    let targetCGY = (screenOriginY + screenHeight) - targetCocoaY

    let newPos = CGPoint(x: currentPos.x, y: targetCGY)
    CGWarpMouseCursorPosition(newPos)
}

// ============================================================================
// Display Configuration Detection
// ============================================================================

func isExternalDisplayPrimary() -> Bool {
    // Check if the main (primary) display is external (not built-in)
    // Returns true when external monitor is primary (clamshell mode or external set as main)
    let mainDisplayID = CGMainDisplayID()
    return CGDisplayIsBuiltin(mainDisplayID) == 0
}

func checkAndUpdateDisplayConfiguration() {
    let externalIsPrimary = isExternalDisplayPrimary()
    let newPosition = externalIsPrimary ? "bottom" : "top"

    if newPosition != barPosition {
        barPosition = newPosition
        if externalIsPrimary {
            triggerSketchyBarEvent("display_external_primary")
            fputs("Display: External monitor is primary - bar moves to bottom\n", stderr)
        } else {
            triggerSketchyBarEvent("display_builtin_primary")
            fputs("Display: Built-in display is primary - bar moves to top\n", stderr)
        }
    }
}

// Display reconfiguration callback
// Called by macOS when display configuration changes (monitor connect/disconnect, primary change)
func displayReconfigurationCallback(
    display: CGDirectDisplayID,
    flags: CGDisplayChangeSummaryFlags,
    userInfo: UnsafeMutableRawPointer?
) {
    // Only respond to meaningful configuration changes
    // BeginConfigurationFlag indicates the START of a change, we wait for completion
    if flags.contains(.beginConfigurationFlag) {
        return
    }

    // Debounce: wait a moment for display list to stabilize
    DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
        checkAndUpdateDisplayConfiguration()
    }
}

// Check accessibility permissions on startup
let hasAccessibilityPermissions = checkAccessibilityPermissions()
if !hasAccessibilityPermissions {
    fputs("Warning: Accessibility permissions not granted. Cursor bouncer disabled.\n", stderr)
    fputs("Grant permissions in System Settings > Privacy & Security > Accessibility\n", stderr)
}

// Register display reconfiguration callback
CGDisplayRegisterReconfigurationCallback(displayReconfigurationCallback, nil)
fputs("Display monitor: Registered for configuration changes\n", stderr)

// Check initial display configuration and trigger appropriate event
checkAndUpdateDisplayConfiguration()

// Main monitoring loop with hysteresis and cursor bouncer
var barIsHidden = false

while true {
    let cursorPos = getCursorPosition()
    let screenTop = getScreenTop()
    let distanceFromTop = screenTop - cursorPos.y

    // Skip bouncer and auto-hide when bar is at bottom (external monitor primary)
    // In this mode, there's no conflict with macOS menu bar
    if barPosition == "bottom" {
        Thread.sleep(forTimeInterval: pollInterval)
        continue
    }

    // BOUNCER: Prevent cursor from entering top zone unless Command is held
    // This runs BEFORE auto-hide logic to prevent accidental menu bar access
    if hasAccessibilityPermissions {
        if distanceFromTop <= bouncerThreshold {
            // Cursor is trying to enter the top zone
            if !isCommandKeyPressed() {
                // Command NOT pressed - bounce cursor back
                bounceCursor(currentPos: cursorPos, screenTop: screenTop)
                // Skip the rest of this iteration since we just moved the cursor
                Thread.sleep(forTimeInterval: pollInterval)
                continue
            }
            // Command IS pressed - allow cursor through, auto-hide will handle it
        }
    }

    // AUTO-HIDE LOGIC (runs if bouncer didn't trigger)
    // Hysteresis logic:
    // - When bar visible: hide if cursor enters top 3px
    // - When bar hidden: show if cursor moves below 44px from top
    // This creates a dead zone (3-44px) that prevents flickering

    if !barIsHidden {
        // Bar is currently visible - check if we should hide it
        if distanceFromTop <= topEdgeThreshold {
            triggerSketchyBarEvent("cursor_at_top")
            barIsHidden = true
        }
    } else {
        // Bar is currently hidden - check if we should show it
        // Only show if cursor is below threshold AND no menu is open
        if distanceFromTop > bottomEdgeActiveThreshold {
            // Check if any menu bar menus are currently open
            if !isMenuBarMenuOpen() {
                triggerSketchyBarEvent("cursor_away_from_top")
                barIsHidden = false
            }
            // If menu is open, keep bar hidden even though cursor is below threshold
        }
    }

    Thread.sleep(forTimeInterval: pollInterval)
}
