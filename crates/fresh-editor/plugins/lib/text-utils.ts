/// <reference path="./fresh.d.ts" />

/**
 * Shared utilities for text plugins (encode decode, text transform, etc.)
 *
 * Provides:
 * - Process selected text wrapper
 *
 * NOTE: These utilities receive the editor instance as a parameter
 * to avoid calling getEditor() at module scope (which causes errors).
 */

// ============================================================================
// Types
// ============================================================================

export async function processSelectedText(
  editor: EditorAPI,
  textProcessorCallback: (
    selectedText: string,
    startSelection: number,
    endSelection: number,
  ) => string,
) {
  try {
    const bufferId = editor.getActiveBufferId();
    const cursorInfo = editor.getPrimaryCursor();

    if (!cursorInfo) {
      throw new Error("No cursor info");
    }

    if (!cursorInfo?.selection) {
      throw new Error("No text selection");
    }

    const startSelection = cursorInfo.selection.start;
    const endSelection = cursorInfo.selection.end;
    const selectedText = await editor.getBufferText(
      bufferId,
      startSelection,
      endSelection,
    );

    const transformedText = textProcessorCallback(
      selectedText,
      startSelection,
      endSelection,
    );

    editor.deleteRange(bufferId, startSelection, endSelection);
    editor.insertText(bufferId, startSelection, transformedText);
  } catch (error) {
    editor.setStatus(`Failed to process selected text: ${error}`);
  }
}
