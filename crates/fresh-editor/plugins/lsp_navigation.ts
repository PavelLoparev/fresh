/// <reference path="../../types/fresh.d.ts" />

interface SymbolItem {
  name: string;
  kind: number;
  startLine: number;
  endLine: number;
  container: string;
}

function getKindLabel(kind: number): string {
  switch (kind) {
    case 1:
      return "file";
    case 2:
      return "mod";
    case 3:
      return "ns";
    case 4:
      return "pkg";
    case 5:
      return "class";
    case 6:
      return "method";
    case 7:
      return "prop";
    case 8:
      return "field";
    case 9:
      return "new";
    case 10:
      return "enum";
    case 11:
      return "iface";
    case 12:
      return "fn";
    case 13:
      return "var";
    case 14:
      return "const";
    case 22:
      return "enum-mem";
    case 23:
      return "struct";
    case 24:
      return "event";
    case 25:
      return "op";
    case 26:
      return "type-param";
    default:
      return "item";
  }
}

const PROMPT_TYPE = "lsp_symbols_list";

let cachedSymbols: SymbolItem[] = [];
let currentSuggestions: SymbolItem[] = [];

async function navigateToSymbol(sym: SymbolItem): Promise<void> {
  const bufferId = editor.getActiveBufferId();

  if (bufferId === null) return;

  const bytePos = await editor.getLineStartPosition(sym.startLine);

  if (bytePos === null) return;

  editor.setBufferCursor(bufferId, bytePos);

  const lineCount = sym.endLine - sym.startLine + 1;

  if (lineCount > 1) {
    editor.executeActions([
      { action: "select_line", count: 1 },
      { action: "select_down", count: lineCount - 2 },
      { action: "select_line_end", count: 1 },
    ]);
  } else {
    editor.executeActions([{ action: "select_line_end", count: 1 }]);
  }

  editor.scrollBufferToLine(bufferId, sym.startLine);
}

async function openSymbolsListHandler(): Promise<void> {
  const bufferId = editor.getActiveBufferId();

  if (bufferId === null) {
    editor.setStatus("No active buffer");

    return;
  }

  const path = editor.getBufferPath(bufferId);

  if (!path) {
    editor.setStatus("Buffer has no file path");

    return;
  }

  editor.setStatus("Loading symbols...");

  try {
    const uri = editor.pathToFileUri(path);
    const result = await editor.sendLspRequest(
      "typescript",
      "textDocument/documentSymbol",
      {
        textDocument: { uri },
      },
    );

    const symbols = parseSymbols(result);

    if (symbols.length === 0) {
      editor.setStatus("No symbols found");

      return;
    }

    cachedSymbols = symbols;
    currentSuggestions = [...symbols];

    const suggestions = symbols.map((sym, i) => ({
      text: `[${getKindLabel(sym.kind)}] ${sym.name}`,
      description: `line ${sym.startLine + 1}`,
      value: String(i),
      disabled: false,
    }));

    editor.startPrompt("Go to symbol: ", PROMPT_TYPE);
    editor.setPromptSuggestions(suggestions);
    editor.setStatus(`${symbols.length} symbols found`);
  } catch (err) {
    editor.setStatus(`Error: ${err}`);
  }
}

registerHandler("goto_lsp_symbol", openSymbolsListHandler);

function parseSymbols(result: unknown): SymbolItem[] {
  const symbols: SymbolItem[] = [];

  if (!result) return symbols;

  if (Array.isArray(result)) {
    for (const item of result) {
      if (typeof item !== "object" || item === null) continue;

      const raw = item as Record<string, unknown>;
      const kind = Number(raw.kind) || 0;
      const name = String(raw.name ?? "");

      if (!name) continue;

      let startLine = 1;
      let endLine = 1;
      let container = "";

      if ("location" in raw && typeof raw.location === "object") {
        const loc = raw.location as Record<string, unknown>;

        if ("range" in loc && typeof loc.range === "object") {
          const range = loc.range as Record<string, unknown>;
          const start = range.start as Record<string, unknown>;
          const end = range.end as Record<string, unknown>;

          startLine = typeof start.line === "number" ? start.line : 0;
          endLine = typeof end.line === "number" ? end.line : startLine;
        }

        container = String(raw.containerName ?? "");
      } else if ("selectionRange" in raw) {
        const selectionRange = raw.selectionRange as Record<string, unknown>;
        const start = selectionRange.start as Record<string, unknown>;
        const end = selectionRange.end as Record<string, unknown>;

        startLine = typeof start.line === "number" ? start.line : 0;
        endLine = typeof end.line === "number" ? end.line : startLine;
      }

      symbols.push({
        name,
        kind,
        startLine,
        endLine,
        container,
      });
    }
  }

  symbols.sort((a, b) => a.startLine - b.startLine);

  return symbols;
}

editor.on("prompt_changed", (args) => {
  if (args.prompt_type !== PROMPT_TYPE) return;

  const query = args.input.toLowerCase();

  if (!query) {
    currentSuggestions = [...cachedSymbols];
  } else {
    currentSuggestions = cachedSymbols.filter((sym) =>
      sym.name.toLowerCase().includes(query),
    );
  }

  const suggestions = currentSuggestions.map((sym, i) => ({
    text: `[${getKindLabel(sym.kind)}] ${sym.name}`,
    description: `line ${sym.startLine + 1}`,
    value: String(i),
    disabled: false,
  }));

  editor.setPromptSuggestions(suggestions);
});

editor.on("prompt_confirmed", async (args) => {
  if (args.prompt_type !== PROMPT_TYPE) return;

  const selectedIndex = args.selected_index;

  if (selectedIndex === null) {
    editor.setStatus("No selection");

    return;
  }

  const sym = currentSuggestions[selectedIndex];

  if (!sym) {
    editor.setStatus("Invalid selection");

    return;
  }

  await navigateToSymbol(sym);
});

editor.on("prompt_selection_changed", async (args) => {
  if (args.prompt_type !== PROMPT_TYPE) return;

  const sym = currentSuggestions[args.selected_index];

  if (!sym) return;

  await navigateToSymbol(sym);
});

editor.registerCommand(
  "Go to LSP Symbol",
  "List document symbols from LSP and navigate to selected",
  "goto_lsp_symbol",
);

editor.setStatus("LSP navigation plugin loaded");
