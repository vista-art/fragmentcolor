// WGSL syntax highlighting for CodeMirror, as a stream parser. It tracks just
// enough state to tell attributes, template brackets, struct fields, and
// definitions apart; everything else is classified token by token.

import { LanguageSupport, StreamLanguage } from "@codemirror/language";

const words = (text) => new Set(text.trim().split(/\s+/));

const DEFINITIONS = words("fn let var const override struct alias");

const CONTROL = words(`
  if else switch case default for loop while break continue continuing return discard
`);

const KEYWORDS = words("enable requires diagnostic const_assert");

const TYPES = words(`
  bool f16 f32 i32 u32 vec2 vec3 vec4 mat2x2 mat2x3 mat2x4 mat3x2 mat3x3 mat3x4 mat4x2 mat4x3 mat4x4
  array atomic ptr sampler sampler_comparison
  texture_1d texture_2d texture_2d_array texture_3d texture_cube texture_cube_array
  texture_multisampled_2d texture_external texture_depth_2d texture_depth_2d_array texture_depth_cube
  texture_depth_cube_array texture_depth_multisampled_2d texture_storage_1d texture_storage_2d
  texture_storage_2d_array texture_storage_3d
`);

// Shorthand aliases such as vec3f, vec4u, mat4x4f, mat3x3h.
const SHORT_TYPE = /^(?:vec[234][fhiu]|mat[234]x[234][fh])$/;

// Address spaces and access modes, as in var<storage, read_write>.
const MODIFIERS = words("function private workgroup uniform storage handle read write read_write");

const TEXEL_FORMAT = /^(?:rgba|rg|r|bgra)(?:8|16|32)(?:unorm|snorm|uint|sint|float)$/;

const BUILTINS = words(`
  abs acos acosh all any arrayLength asin asinh atan atan2 atanh
  atomicAdd atomicAnd atomicCompareExchangeWeak atomicExchange atomicLoad atomicMax atomicMin
  atomicOr atomicStore atomicSub atomicXor bitcast ceil clamp cos cosh countLeadingZeros
  countOneBits countTrailingZeros cross degrees determinant distance dot dot4I8Packed dot4U8Packed
  dpdx dpdxCoarse dpdxFine dpdy dpdyCoarse dpdyFine exp exp2 extractBits faceForward
  firstLeadingBit firstTrailingBit floor fma fract frexp fwidth fwidthCoarse fwidthFine insertBits
  inverseSqrt ldexp length log log2 max min mix modf normalize pack2x16float pack2x16snorm
  pack2x16unorm pack4x8snorm pack4x8unorm pack4xI8 pack4xI8Clamp pack4xU8 pack4xU8Clamp pow
  quantizeToF16 radians reflect refract reverseBits round saturate select sign sin sinh smoothstep
  sqrt step storageBarrier tan tanh textureBarrier textureDimensions textureGather
  textureGatherCompare textureLoad textureNumLayers textureNumLevels textureNumSamples textureSample
  textureSampleBaseClampToEdge textureSampleBias textureSampleCompare textureSampleCompareLevel
  textureSampleGrad textureSampleLevel textureStore transpose trunc unpack2x16float unpack2x16snorm
  unpack2x16unorm unpack4x8snorm unpack4x8unorm unpack4xI8 unpack4xU8 workgroupBarrier
  workgroupUniformLoad subgroupAdd subgroupAll subgroupAny subgroupBallot subgroupBroadcast
  subgroupBroadcastFirst subgroupElect subgroupMax subgroupMin subgroupMul subgroupShuffle
  subgroupShuffleDown subgroupShuffleUp subgroupShuffleXor quadBroadcast quadSwapDiagonal
  quadSwapX quadSwapY
`);

// Hex and decimal literals, floats with exponents, and the f, h, i, u suffixes.
const NUMBER =
  /^(?:0[xX](?:[0-9a-fA-F]+\.?[0-9a-fA-F]*|\.[0-9a-fA-F]+)(?:[pP][+-]?[0-9]+)?[fhiu]?|(?:[0-9]+\.[0-9]*|\.[0-9]+)(?:[eE][+-]?[0-9]+)?[fh]?|[0-9]+(?:[eE][+-]?[0-9]+)?[fhiu]?)/;

const IDENTIFIER = /^[A-Za-z_][A-Za-z0-9_]*/;

const OPERATOR = /^(?:->|<<=|>>=|&&|\|\||==|!=|<=|>=|<<|>>|\+\+|--|[-+*/%&|^]=|[-+*/%&|^~!<>=])/;

// Attributes whose arguments are names from a fixed list, as in @builtin(position).
const ENUM_ATTRIBUTES = words("builtin interpolate diagnostic");

// Names that open a template list when a `<` follows them directly.
function isTemplated(word) {
  return TYPES.has(word) || word === "var";
}

function readComment(stream, state) {
  while (!stream.eol()) {
    if (stream.match("/*")) {
      state.comment++;
    } else if (stream.match("*/")) {
      state.comment--;
      if (state.comment === 0) break;
    } else {
      stream.next();
    }
  }
  return "blockComment";
}

function readWord(stream, state, word) {
  const previous = state.previous;
  if (state.attribute > 0 && state.enumerant) return "atom";
  if (word === "true" || word === "false") return "bool";
  if (DEFINITIONS.has(word)) {
    state.pending = word;
    return "definitionKeyword";
  }
  if (CONTROL.has(word)) return "controlKeyword";
  if (KEYWORDS.has(word)) return "keyword";
  if (TYPES.has(word) || SHORT_TYPE.test(word)) return "typeName";
  if (state.template > 0 && (MODIFIERS.has(word) || TEXEL_FORMAT.test(word))) return "modifier";
  if (previous === ".") return "propertyName";
  if (previous === "fn") return "variableName.function.definition";
  if (previous === "struct" || previous === "alias") return "typeName.definition";
  if (previous === "let" || previous === "var" || previous === "const" || previous === "override") {
    return "variableName.definition";
  }
  if (stream.match(/^\s*\(/, false)) {
    return BUILTINS.has(word) ? "variableName.function.standard" : "variableName.function";
  }
  if (stream.match(/^\s*:(?!:)/, false)) {
    return state.struct > 0 && state.braces === state.struct ? "propertyName.definition" : "variableName.definition";
  }
  if (/^[A-Z][A-Z0-9_]+$/.test(word)) return "variableName.constant";
  if (/^[A-Z]/.test(word)) return "typeName";
  return "variableName";
}

function readPunctuation(state, char) {
  if (char === "(") {
    if (state.attribute > 0) {
      state.attribute++;
    } else if (state.previous?.startsWith("@")) {
      state.attribute = 1;
      state.enumerant = ENUM_ATTRIBUTES.has(state.previous.slice(1));
    }
    return "paren";
  }
  if (char === ")") {
    if (state.attribute > 0) state.attribute--;
    return "paren";
  }
  if (char === "{") {
    state.braces++;
    if (state.pending === "struct") state.struct = state.braces;
    state.pending = null;
    return "brace";
  }
  if (char === "}") {
    if (state.braces === state.struct) state.struct = 0;
    state.braces = Math.max(0, state.braces - 1);
    return "brace";
  }
  if (char === "[" || char === "]") return "squareBracket";
  if (char === ";") state.pending = null;
  if (char === ".") return "derefOperator";
  if (char === "," || char === ";" || char === ":") return "separator";
  return "punctuation";
}

function readToken(stream, state) {
  if (state.comment > 0) return readComment(stream, state);

  const touching = stream.start === 0 || !/\s/.test(stream.string.charAt(stream.start - 1));
  if (stream.eatSpace()) return null;
  if (stream.match("//")) {
    stream.skipToEnd();
    return "lineComment";
  }
  if (stream.match("/*")) {
    state.comment = 1;
    return readComment(stream, state);
  }

  if (stream.eat("@")) {
    const name = stream.match(IDENTIFIER);
    if (name) {
      state.previous = `@${name[0]}`;
      return "attributeName";
    }
    return "operator";
  }

  if (stream.match(NUMBER)) {
    state.previous = "number";
    return "number";
  }

  const word = stream.match(IDENTIFIER);
  if (word) {
    const style = readWord(stream, state, word[0]);
    state.previous = word[0];
    return style;
  }

  // A `<` right after a templated type or `var` opens a template list.
  if (stream.peek() === "<" && touching && isTemplated(state.previous)) {
    stream.next();
    state.template++;
    state.previous = "<";
    return "angleBracket";
  }
  if (stream.peek() === ">" && state.template > 0) {
    stream.next();
    state.template--;
    state.previous = ">";
    return "angleBracket";
  }

  if (stream.match(OPERATOR)) {
    state.previous = stream.current();
    return "operator";
  }

  const char = stream.next();
  const style = readPunctuation(state, char);
  state.previous = char;
  return style;
}

const parser = {
  name: "wgsl",
  startState: () => ({
    comment: 0,
    braces: 0,
    struct: 0,
    template: 0,
    attribute: 0,
    enumerant: false,
    pending: null,
    previous: null,
  }),
  token: readToken,
  indent(state, textAfter, context) {
    const depth = Math.max(0, state.braces - (/^\s*}/.test(textAfter) ? 1 : 0));
    return depth * context.unit;
  },
  languageData: {
    commentTokens: { line: "//", block: { open: "/*", close: "*/" } },
    closeBrackets: { brackets: ["(", "[", "{"] },
    indentOnInput: /^\s*}$/,
    autocomplete: completeWords,
  },
};

const COMPLETIONS = [
  ...[...DEFINITIONS, ...CONTROL, ...KEYWORDS, "true", "false"].map((label) => ({ label, type: "keyword" })),
  ...[...TYPES, ...MODIFIERS].map((label) => ({ label, type: "type" })),
  ...[...BUILTINS].map((label) => ({ label, type: "function" })),
];

// Completion source for keywords, types, and builtin functions. It reads
// only the context object, so the autocomplete package stays optional.
function completeWords(context) {
  const word = context.matchBefore(/[A-Za-z_][A-Za-z0-9_]*/);
  if (!word || (word.from === word.to && !context.explicit)) return null;
  return { from: word.from, options: COMPLETIONS, validFor: /^[A-Za-z0-9_]*$/ };
}

export const wgslLanguage = StreamLanguage.define(parser);

export function wgsl() {
  return new LanguageSupport(wgslLanguage);
}
