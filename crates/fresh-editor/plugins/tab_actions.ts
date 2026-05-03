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
  "%cmd.close_others",
  "%cmd.close_others_desc",
  "close_other_buffers"
);

editor.registerCommand(
  "%cmd.close_all",
  "%cmd.close_all_desc",
  "close_all_buffers"
);

editor.registerCommand(
  "%cmd.close_to_right",
  "%cmd.close_to_right_desc",
  "close_buffers_to_right"
);

editor.registerCommand(
  "%cmd.close_to_left",
  "%cmd.close_to_left_desc",
  "close_buffers_to_left"
);

editor.registerCommand(
  "%cmd.move_left",
  "%cmd.move_left_desc",
  "move_tab_left"
);

editor.registerCommand(
  "%cmd.move_right",
  "%cmd.move_right_desc",
  "move_tab_right"
);

editor.setStatus(editor.t("status.plugin_loaded"));
