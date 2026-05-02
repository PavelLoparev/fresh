/// <reference path="./lib/fresh.d.ts" />

/**
 * Text Transform Plugin for Fresh Editor
 *
 * Provides text transform functionality for selected text.
 */

import { processSelectedText } from "./lib/text-utils";

const editor = getEditor();

function stringToTitleCaseActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    const titleBoundary = new RegExp("(^|[^\\p{L}\\p{N}']|((^|\\P{L})'))\\p{L}",  "gmu");

    return selectedText
      .toLocaleLowerCase()
      .replace(titleBoundary, (b) => b.toLocaleUpperCase());
  });
}

function stringToKebabCaseActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    const caseBoundary = new RegExp("(\\p{Ll})(\\p{Lu})", "gmu");
    const singleLetters = new RegExp("(\\p{Lu}|\\p{N})(\\p{Lu}\\p{Ll})", "gmu");
    const underscoreBoundary = new RegExp("(\\S)(_)(\\S)", "gm");

    return selectedText
      .replace(underscoreBoundary, "$1-$3")
      .replace(caseBoundary, "$1-$2")
      .replace(singleLetters, "$1-$2")
      .toLocaleLowerCase();
  });
}

function stringToSnakeCaseActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    const caseBoundary = new RegExp("(\\p{Ll})(\\p{Lu})", "gmu");
    const singleLetters = new RegExp("(\\p{Lu}|\\p{N})(\\p{Lu})(\\p{Ll})", "gmu");

    return selectedText
      .replace(caseBoundary, "$1_$2")
      .replace(singleLetters, "$1_$2$3")
      .toLocaleLowerCase();
  });
}

function stringToCamelCaseActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    const singleLineWordBoundary = new RegExp("[_\\s-]+", "gm");
    const multiLineWordBoundary = new RegExp("[_-]+", "gm");
    const validWordStart = new RegExp("^(\\p{Lu}[^\\p{Lu}])", "gmu");

    const wordBoundary = /\r\n|\r|\n/.test(selectedText)
      ? multiLineWordBoundary
      : singleLineWordBoundary;

    const words = selectedText.split(wordBoundary);
    const firstWord = words
      .shift()
      ?.replace(validWordStart, (start: string) => start.toLocaleLowerCase());

    return firstWord + words
      .map((word: string) => word.substring(0, 1).toLocaleUpperCase() + word.substring(1))
      .join("");
  });
}

function stringToPascalCaseActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    const wordBoundary = new RegExp("[_ \\t-]", "gm");
    const wordBoundaryToMaintain = new RegExp("(?<=\\.)", "gm");
    const upperCaseWordMatcher = new RegExp("^\\p{Lu}+$", "mu");
    const wordsWithMaintainBoundaries = selectedText.split(wordBoundaryToMaintain);
    const words = wordsWithMaintainBoundaries
      .map((word) => word.split(wordBoundary))
      .flat();

    return words
      .map((word) => {
        const normalizedWord = word.charAt(0).toLocaleUpperCase() + word.slice(1);
        const isAllCaps = normalizedWord.length > 1 && upperCaseWordMatcher.test(normalizedWord);

        if (isAllCaps) {
          return normalizedWord.charAt(0) + normalizedWord.slice(1).toLocaleLowerCase();
        }

        return normalizedWord;
      })
      .join("");
  });
}

registerHandler("string_to_kebab_case", stringToKebabCaseActionHandler);
registerHandler("string_to_title_case", stringToTitleCaseActionHandler);
registerHandler("string_to_snake_case", stringToSnakeCaseActionHandler);
registerHandler("string_to_camel_case", stringToCamelCaseActionHandler);
registerHandler("string_to_pascal_case", stringToPascalCaseActionHandler);

editor.registerCommand(
  "Transform to kebab-case",
  "Transform selected string to kebab-case",
  "string_to_kebab_case",
);

editor.registerCommand(
  "Transform to Title Case",
  "Transform selected string to Title Case",
  "string_to_title_case",
);

editor.registerCommand(
  "Transform to snake_case",
  "Transform selected string to snake_case",
  "string_to_snake_case",
);

editor.registerCommand(
  "Transform to camelCase",
  "Transform selected string to camelCase",
  "string_to_camel_case",
);

editor.registerCommand(
  "Transform to PascalCase",
  "Transform selected string to PascalCase",
  "string_to_pascal_case",
);

editor.setStatus("Text Transform plugin loaded");
