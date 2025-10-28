#!/usr/bin/swift

import Foundation
import CoreGraphics
import AppKit

// Configuration
// NOTE: These values are tuned for MacBook Air at 1710x1112 resolution
// May need adjustment for different display resolutions/DPI settings
// TODO: Consider dynamic resolution detection for multi-display setups
let topEdgeThreshold: CGFloat = 1.0         // Hide bar when cursor within 1px of top (tight)
let bottomEdgeActiveThreshold: CGFloat = 30.0  // Show bar when cursor moves below 30px from top (loose)
let pollInterval: TimeInterval = 0.1
let sketchybarPath = "/opt/homebrew/bin/sketchybar"

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

func triggerSketchyBarEvent(_ eventName: String) {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: sketchybarPath)
    process.arguments = ["--trigger", eventName]

    // Redirect output to /dev/null
    process.standardOutput = FileHandle.nullDevice
    process.standardError = FileHandle.nullDevice

    try? process.run()
}

// Main monitoring loop with hysteresis
var barIsHidden = false

while true {
    let cursorPos = getCursorPosition()
    let screenTop = getScreenTop()
    let distanceFromTop = screenTop - cursorPos.y

    // Hysteresis logic:
    // - When bar visible: hide if cursor enters top 5px
    // - When bar hidden: show if cursor moves below 40px from top
    // This creates a dead zone (5-40px) that prevents flickering

    if !barIsHidden {
        // Bar is currently visible - check if we should hide it
        if distanceFromTop <= topEdgeThreshold {
            triggerSketchyBarEvent("cursor_at_top")
            barIsHidden = true
        }
    } else {
        // Bar is currently hidden - check if we should show it
        if distanceFromTop > bottomEdgeActiveThreshold {
            triggerSketchyBarEvent("cursor_away_from_top")
            barIsHidden = false
        }
    }

    Thread.sleep(forTimeInterval: pollInterval)
}
