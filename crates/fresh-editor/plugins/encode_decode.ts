/// <reference path="./lib/fresh.d.ts" />

/**
 * Text Encode Decode Plugin for Fresh Editor
 *
 * Provides text encoding and decoding functionality for selected text.
 */

import { processSelectedText } from "./lib/text-utils";

const editor = getEditor();

function base64Encode(input: string): string {
  const keyStr =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
  let output = "";
  let chr1, chr2, chr3, enc1, enc2, enc3, enc4;
  let i = 0;

  const utf8Text = unescape(encodeURIComponent(input));

  while (i < utf8Text.length) {
    chr1 = utf8Text.charCodeAt(i++);
    chr2 = utf8Text.charCodeAt(i++);
    chr3 = utf8Text.charCodeAt(i++);

    enc1 = chr1 >> 2;
    enc2 = ((chr1 & 3) << 4) | (chr2 >> 4);
    enc3 = ((chr2 & 15) << 2) | (chr3 >> 6);
    enc4 = chr3 & 63;

    if (isNaN(chr2)) {
      enc3 = enc4 = 64;
    } else if (isNaN(chr3)) {
      enc4 = 64;
    }

    output +=
      keyStr.charAt(enc1) +
      keyStr.charAt(enc2) +
      keyStr.charAt(enc3) +
      keyStr.charAt(enc4);
  }

  return output;
}

function base64Decode(input: string): string {
  const keyStr =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
  let output = "";
  let chr1, chr2, chr3;
  let enc1, enc2, enc3, enc4;
  let i = 0;

  // Remove invalid characters
  input = input.replace(/[^A-Za-z0-9\+\/\=]/g, "");

  while (i < input.length) {
    enc1 = keyStr.indexOf(input.charAt(i++));
    enc2 = keyStr.indexOf(input.charAt(i++));
    enc3 = keyStr.indexOf(input.charAt(i++));
    enc4 = keyStr.indexOf(input.charAt(i++));

    chr1 = (enc1 << 2) | (enc2 >> 4);
    chr2 = ((enc2 & 15) << 4) | (enc3 >> 2);
    chr3 = ((enc3 & 3) << 6) | enc4;

    output += String.fromCharCode(chr1);

    if (enc3 !== 64) {
      output += String.fromCharCode(chr2);
    }
    if (enc4 !== 64) {
      output += String.fromCharCode(chr3);
    }
  }

  return decodeURIComponent(escape(output));
}

function jsonByteArrayToHexString(input: string): string {
  try {
    const parsed = JSON.parse(input);

    // Validate input is an array
    if (!Array.isArray(parsed)) {
      throw new Error("Input is not a JSON array");
    }

    // Validate all elements are integers between 0-255
    for (let i = 0; i < parsed.length; i++) {
      const element = parsed[i];
      if (!Number.isInteger(element)) {
        throw new Error(`Element at index ${i} is not an integer: ${element}`);
      }
      if (element < 0 || element > 255) {
        throw new Error(
          `Element at index ${i} is out of byte range (0-255): ${element}`,
        );
      }
    }

    // Convert each byte to 2-digit hex and concatenate
    let output = "";
    for (let i = 0; i < parsed.length; i++) {
      const hexElement = parsed[i].toString(16).padStart(2, "0");
      output += hexElement;
    }

    return output;
  } catch (error) {
    if (error instanceof SyntaxError) {
      throw new Error("Invalid JSON input");
    }
    throw error;
  }
}

function hexStringToJsonByteArray(input: string): string {
  // Remove any whitespace
  const cleanedInput = input.replace(/\s/g, "");

  // Validate input is a valid hex string
  if (!/^[0-9a-fA-F]+$/.test(cleanedInput)) {
    throw new Error("Input contains invalid hex characters");
  }

  // Validate input length is even (each byte is 2 hex chars)
  if (cleanedInput.length % 2 !== 0) {
    throw new Error("Hex string must have an even number of characters");
  }

  // Convert each pair of hex characters to a byte
  const byteArray = [];
  for (let i = 0; i < cleanedInput.length; i += 2) {
    const hexByte = cleanedInput.substr(i, 2);
    const byte = parseInt(hexByte, 16);
    if (isNaN(byte)) {
      throw new Error(`Invalid hex byte: ${hexByte}`);
    }
    byteArray.push(byte);
  }

  // Return as JSON array
  return JSON.stringify(byteArray);
}

function stringToJsonString(input: string): string {
  return JSON.stringify(input);
}

function jsonStringToString(input: string): string {
  try {
    const parsed = JSON.parse(input);
    // Ensure we return a string (JSON.parse can return other types)
    if (typeof parsed === "string") {
      return parsed;
    } else {
      // If it's not a string, convert it to string representation
      return String(parsed);
    }
  } catch (error) {
    throw new Error(`Invalid JSON string: ${error.message}`);
  }
}

function stringToUriEncoded(input: string): string {
  return encodeURI(input);
}

function uriEncodedToString(input: string): string {
  return decodeURI(input);
}

function stringToUriComponentEncoded(input: string): string {
  return encodeURIComponent(input);
}

function uriComponentEncodedToString(input: string): string {
  return decodeURIComponent(input);
}

function stringToBase64ActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    return base64Encode(selectedText)
  });
}

function base64ToStringActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    // Validate that input looks like Base64 before attempting to decode
    if (!/^[A-Za-z0-9+/]*={0,2}$/.test(selectedText.replace(/\s/g, ""))) {
      throw new Error("Invalid Base64 input");
    }

    return base64Decode(selectedText)
  });
}

function jsonByteArrayToHexStringActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    return jsonByteArrayToHexString(selectedText);
  });
}

function hexStringToJsonByteArrayActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    return hexStringToJsonByteArray(selectedText);
  });
}

function stringToJsonStringActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    return stringToJsonString(selectedText)
  });
}

function jsonStringToStringActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    return jsonStringToString(selectedText);
  });
}

function stringToUriEncodedActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    return stringToUriEncoded(selectedText);
  });
}

function uriEncodedToStringActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    return uriEncodedToString(selectedText);
  });
}

function stringToUriComponentEncodedActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    return stringToUriComponentEncoded(selectedText);
  });
}

function uriComponentEncodedToStringActionHandler(): void {
  processSelectedText(editor, (selectedText) => {
    return uriComponentEncodedToString(selectedText);
  });
}

registerHandler("string_to_base64", stringToBase64ActionHandler);
registerHandler("base64_to_string", base64ToStringActionHandler);
registerHandler("json_byte_array_to_hex_string", jsonByteArrayToHexStringActionHandler);
registerHandler("hex_string_to_json_byte_array", hexStringToJsonByteArrayActionHandler);
registerHandler("string_to_json_string", stringToJsonStringActionHandler);
registerHandler("json_string_to_string", jsonStringToStringActionHandler);
registerHandler("string_to_uri_encoded", stringToUriEncodedActionHandler);
registerHandler("uri_encoded_to_string", uriEncodedToStringActionHandler);
registerHandler(
  "string_to_uri_component_encoded",
  stringToUriComponentEncodedActionHandler,
);
registerHandler(
  "uri_component_encoded_to_string",
  uriComponentEncodedToStringActionHandler,
);

editor.setStatus(editor.t("status.plugin_ready"));

editor.registerCommand(
  "%cmd.string_to_base64",
  "%cmd.string_to_base64_desc",
  "string_to_base64",
);

editor.registerCommand(
  "%cmd.base64_to_string",
  "%cmd.base64_to_string_desc",
  "base64_to_string",
);

editor.registerCommand(
  "%cmd.json_byte_array_to_hex_string",
  "%cmd.json_byte_array_to_hex_string_desc",
  "json_byte_array_to_hex_string",
);

editor.registerCommand(
  "%cmd.hex_string_to_json_byte_array",
  "%cmd.hex_string_to_json_byte_array_desc",
  "hex_string_to_json_byte_array",
);

editor.registerCommand(
  "%cmd.string_to_json_string",
  "%cmd.string_to_json_string_desc",
  "string_to_json_string",
);

editor.registerCommand(
  "%cmd.json_string_to_string",
  "%cmd.json_string_to_string_desc",
  "json_string_to_string",
);

editor.registerCommand(
  "%cmd.string_to_uri_encoded",
  "%cmd.string_to_uri_encoded_desc",
  "string_to_uri_encoded",
);

editor.registerCommand(
  "%cmd.uri_encoded_to_string",
  "%cmd.uri_encoded_to_string_desc",
  "uri_encoded_to_string",
);

editor.registerCommand(
  "%cmd.string_to_uri_component_encoded",
  "%cmd.string_to_uri_component_encoded_desc",
  "string_to_uri_component_encoded",
);

editor.registerCommand(
  "%cmd.uri_component_encoded_to_string",
  "%cmd.uri_component_encoded_to_string_desc",
  "uri_component_encoded_to_string"
);
