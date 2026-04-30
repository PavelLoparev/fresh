/// <reference path="./lib/fresh.d.ts" />
const editor = getEditor();

async function goto_line_with_selection_handler(): Promise<void> {
  editor.executeActions([
    { action: "set_mark", count: 1 },
    { action: "goto_line", count: 1 },
  ]);
}

registerHandler("goto_line_with_selection", goto_line_with_selection_handler);

editor.registerCommand(
  "Go to Line with Selection",
  "Select from current position to target line",
  "goto_line_with_selection",
);
