// Light syntax highlighting for the generated Rust, Python, Swift, and Kotlin
// tabs: keywords, strings, numbers, comments, and type names. They are read
// only, so a token stream is enough.

import { LanguageSupport, StreamLanguage } from "@codemirror/language";

const KEYWORDS = {
  rust: "as async await const crate else enum fn for if impl in let loop match mod move mut pub ref return self Self static struct trait true false type use where while",
  python: "and as async await class def elif else False for from if import in is lambda None not or pass return True while with",
  swift: "as async await case class else enum false for func guard if import in init let nil private return self static struct throws true try var while",
  kotlin: "as class else false for fun if import in is null object private return this true val var when while",
};

function language(name, { hash = false } = {}) {
  const keywords = new Set(KEYWORDS[name].split(" "));
  return StreamLanguage.define({
    name,
    startState: () => ({ block: false }),
    token(stream, state) {
      if (state.block) {
        if (stream.skipTo("*/")) {
          stream.next();
          stream.next();
          state.block = false;
        } else stream.skipToEnd();
        return "comment";
      }
      if (stream.eatSpace()) return null;
      if (hash ? stream.match("#") : stream.match("//")) {
        stream.skipToEnd();
        return "comment";
      }
      if (!hash && stream.match("/*")) {
        state.block = true;
        return "comment";
      }
      if (stream.match(/^"""/)) {
        stream.skipTo('"""') ? stream.match('"""') : stream.skipToEnd();
        return "string";
      }
      if (stream.match(/^r?"(?:[^"\\]|\\.)*"?/) || stream.match(/^'(?:[^'\\]|\\.)*'?/)) return "string";
      if (stream.match(/^-?\d[\d_]*(\.\d+)?([eE][-+]?\d+)?[fu]?\w*/)) return "number";
      if (stream.match(/^[A-Za-z_]\w*!?/)) {
        const word = stream.current();
        if (keywords.has(word)) return "keyword";
        if (word.endsWith("!")) return "macroName";
        if (/^[A-Z]/.test(word)) return "typeName";
        return stream.peek() === "(" ? "variableName.function" : "variableName";
      }
      stream.next();
      return null;
    },
    languageData: { commentTokens: { line: hash ? "#" : "//" } },
  });
}

const LANGUAGES = {
  rust: () => language("rust"),
  python: () => language("python", { hash: true }),
  swift: () => language("swift"),
  kotlin: () => language("kotlin"),
};

export function highlight(id) {
  const make = LANGUAGES[id];
  return make ? new LanguageSupport(make()) : [];
}
