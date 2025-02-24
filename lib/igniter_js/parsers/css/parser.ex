defmodule IgniterJs.Parsers.CSS.Parser do
  def add_value() do
    css_code = """
    /* This is a top comment1 */
    .hide-scrollbar {
        /* Inner comment */
        -ms-overflow-style: none; /* Internet Explorer 10+ */
        scrollbar-width: none; /* Firefox */
    }
    /* This is a top comment2 */
    .hide-scrollbar::-webkit-scrollbar {
        display: none; /* Safari and Chrome */
    }
    .hide-scrollbar:something {
        display: flex;
    }
    /* This is a top comment3 */
    .other-class {
        color: red;
    }

    /* This is a bot comment1 */
    """

    {result, _globals} =
      Pythonx.eval(
        """
        import tinycss2

        def modify_css(css):
            css = css.decode('utf-8') if isinstance(css, bytes) else css
            rules = tinycss2.parse_stylesheet(css, skip_whitespace=False, skip_comments=False)
            found_hide_scrollbar = False
            modified_css = ""

            for rule in rules:
                if rule.type == "qualified-rule":
                    selector = tinycss2.serialize(rule.prelude).strip()
                    declarations = tinycss2.parse_declaration_list(rule.content, skip_whitespace=False, skip_comments=False)

                    if selector == ".hide-scrollbar":
                        found_hide_scrollbar = True
                        has_display_none = any(
                            node.type == "declaration"
                            and node.name == "display"
                            and tinycss2.serialize(node.value).strip() == "none"
                            for node in declarations
                        )

                        if not has_display_none:
                            if declarations and declarations[-1].type != "whitespace":
                                declarations.append(tinycss2.ast.WhitespaceToken(line=0, column=0, value='\\n    '))
                            declarations.append(
                                tinycss2.ast.Declaration(
                                    name="display",
                                    value=[
                                        tinycss2.ast.WhitespaceToken(value=" ", line=0, column=0),
                                        tinycss2.ast.IdentToken(value="none", line=0, column=0)
                                    ],
                                    important=False,
                                    line=0,
                                    column=0,
                                    lower_name="display",
                                )
                            )

                    serialized_content = tinycss2.serialize(declarations).strip()
                    serialized_content = "\\n".join("    " + line.strip() for line in serialized_content.splitlines() if line.strip())
                    formatted_rule = f"{selector} {{\\n{serialized_content}\\n}}\\n"
                    modified_css += formatted_rule

                else:
                    modified_css += tinycss2.serialize([rule])

            if not found_hide_scrollbar:
                modified_css += "\\n.hide-scrollbar {\\n    display: none;\\n}\\n"

            return modified_css.strip()


        modify_css(css_code)
        """,
        %{"css_code" => css_code}
      )

    Pythonx.decode(result)
  end
end
