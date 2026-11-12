# Menus, windows and navigation

- Choose MenuBarExtra, an AppKit status item or the existing bridge according to the repository's current behavior and supported target. A functioning AppKit integration is not a SwiftUI defect.
- Route menu actions, Raycast actions and window controls to the same domain operation. Reopening a window must not start a new session or create duplicate writers. Preserve app activation policy, login registration and configured runtime identity.
- Treat window size and available display space as changeable. Use flexible frames, scrolling or a supported ViewThatFits alternative where content would otherwise clip. Essential actions must remain reachable in short/narrow windows and with longer text.
- Do not position windows against one assumed screen. Resolve the appropriate screen from the window/status-item context; account for multiple displays and an anchor disappearing.
- Keep navigation state above layout branches whose identity changes. Use scene-local state for independent windows. Prefer supported NavigationStack/SplitView APIs for new navigation when appropriate; leave existing navigation intact unless it causes an in-scope problem.
- Register typed destinations once per type per stack. Avoid competing destination mechanisms. When presentation depends on an optional selected item, consider item-driven sheets to avoid separate Boolean/value state getting out of sync.
- Prefer native sheets, popovers, alerts and semantic toolbar groups. Reserve space using supported system layout rather than guessed safe-area offsets. Keep confirmation attached to a stable view that survives menu dismissal.
- Verify open, close, reopen, cancel and completion/error states. A menu-only action, Settings window and Quit behavior can differ intentionally; preserve those differences in tests.
