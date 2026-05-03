/// <reference path="../../types/fresh.d.ts" />

const editor = getEditor();

/**
 * Tabs actions plugin
 */

function closeOtherBuffers() : void {
  editor.closeOtherBuffersInSplit(editor.getActiveBufferId(), editor.getActiveSplitId());
}

function closeAllBuffers() : void {
  editor.closeAllBuffersInSplit(editor.getActiveSplitId());
}

function closeBuffersToRight() : void {
  editor.closeBuffersToRightInSplit(editor.getActiveBufferId(), editor.getActiveSplitId());
}

function closeBuffersToLeft() : void {
  editor.closeBuffersToLeftInSplit(editor.getActiveBufferId(), editor.getActiveSplitId());
}

function moveTabLeft() : void {
  editor.moveTabLeft();
}

function moveTabRight() : void {
  editor.moveTabRight();
}

registerHandler("close_other_buffers", closeOtherBuffers);
registerHandler("close_all_buffers", closeAllBuffers);
registerHandler("close_buffers_to_right", closeBuffersToRight);
registerHandler("close_buffers_to_left", closeBuffersToLeft);
registerHandler("move_tab_left", moveTabLeft);
registerHandler("move_tab_right", moveTabRight);

editor.registerCommand(
  "Close Other Tabs",
  "Close other tabs in the current split",
  "close_other_buffers"
);

editor.registerCommand(
  "Close All Tabs",
  "Close all tabs in the current split",
  "close_all_buffers"
);

editor.registerCommand(
  "Close Tabs To Right",
  "Close tabs to right in the current split",
  "close_buffers_to_right"
);

editor.registerCommand(
  "Close Tabs To Left",
  "Close tabs to left in the current split",
  "close_buffers_to_left"
);

editor.registerCommand(
  "Move Tab Left",
  "Move active tab on the active split left",
  "move_tab_left"
);

editor.registerCommand(
  "Move Tab Right",
  "Move active tab on the active split right",
  "move_tab_right"
);

editor.setStatus("Tab actions plugin loaded");
