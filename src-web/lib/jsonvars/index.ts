import { LRLanguage, LanguageSupport, foldNodeProp, foldInside, indentNodeProp, continuedIndent } from "@codemirror/language";
import { styleTags, tags as t } from "@lezer/highlight";
import { parser } from "./jsonvars.grammar";

/**
 * JSON language that understands `{{variable}}` placeholders as values and
 * property names. Used for editable request bodies; plain JSON responses
 * keep using @codemirror/lang-json.
 */
export const jsonVarsLanguage = LRLanguage.define({
  name: "json",
  parser: parser.configure({
    props: [
      indentNodeProp.add({
        Object: continuedIndent({ except: /^\s*\}/ }),
        Array: continuedIndent({ except: /^\s*\]/ }),
      }),
      foldNodeProp.add({
        "Object Array": foldInside,
      }),
      styleTags({
        String: t.string,
        Number: t.number,
        "True False": t.bool,
        PropertyName: t.propertyName,
        Null: t.null,
        Variable: t.special(t.variableName),
        ",": t.separator,
        "[ ]": t.squareBracket,
        "{ }": t.brace,
      }),
    ],
  }),
  languageData: {
    closeBrackets: { brackets: ["[", "{", '"'] },
    indentOnInput: /^\s*[}\]]$/,
  },
});

export function jsonVars() {
  return new LanguageSupport(jsonVarsLanguage);
}
