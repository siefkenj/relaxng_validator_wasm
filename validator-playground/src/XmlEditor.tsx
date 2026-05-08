import { useMemo } from "react";
import CodeMirror from "@uiw/react-codemirror";
import { xml } from "@codemirror/lang-xml";
import { linter, type Diagnostic } from "@codemirror/lint";
import type { ValidationResponse } from "./validatorWorkerApi.ts";

type Props = {
    value: string;
    errors: ValidationResponse["errors"];
    onChange: (value: string) => void;
};

function clampSpan(start: number, end: number, docLength: number) {
    const clampedStart = Math.max(0, Math.min(start, docLength));
    const clampedEnd = Math.max(clampedStart + 1, Math.min(end, docLength));
    return { from: clampedStart, to: clampedEnd };
}

function toDiagnosticMessage(error: ValidationResponse["errors"][number]) {
    switch (error.type) {
        case "NotAllowed":
            return "Unexpected token.";
        case "UndefinedNamespacePrefix":
            return `Undefined namespace prefix: ${error.prefix.text}`;
        case "UndefinedEntity":
            return `Undefined entity: ${error.name}`;
        case "InvalidOrUnclosedEntity":
            return "Invalid or unclosed entity.";
        case "Xml":
            return error.message;
        case "TextBufferOverflow":
            return "Text buffer overflow while parsing XML.";
        case "TooManyPatterns":
            return "Schema exceeded supported pattern complexity.";
        default:
            return "Schema validation error.";
    }
}

function appendSuggestionRow(
    container: HTMLElement,
    label: string,
    suggestions: string[],
    formatter: (value: string) => string,
) {
    if (suggestions.length === 0) {
        return;
    }

    const row = document.createElement("div");
    row.append(`${label}: `);

    suggestions.forEach((suggestion, index) => {
        if (index > 0) {
            row.append(", ");
        }

        const code = document.createElement("code");
        code.textContent = formatter(suggestion);
        row.append(code);
    });

    container.append(row);
}

function renderDiagnosticMessage(error: ValidationResponse["errors"][number]) {
    if (error.type !== "NotAllowed") {
        return undefined;
    }

    return () => {
        const wrapper = document.createElement("div");

        const header = document.createElement("div");
        header.textContent = "Unexpected token.";
        wrapper.append(header);

        const hasElementSuggestions = error.expected_elements.length > 0;
        const hasAttributeSuggestions = error.expected_attributes.length > 0;

        if (!hasElementSuggestions && !hasAttributeSuggestions) {
            return wrapper;
        }

        appendSuggestionRow(
            wrapper,
            "Expected elements",
            error.expected_elements,
            (value) => `<${value}>`,
        );
        appendSuggestionRow(
            wrapper,
            "Expected attributes",
            error.expected_attributes,
            (value) => `${value}="..."`,
        );

        return wrapper;
    };
}

function toErrorSpan(error: ValidationResponse["errors"][number]) {
    switch (error.type) {
        case "NotAllowed": {
            const tokenSpan =
                "span" in error.token
                    ? error.token.span
                    : error.token.type === "Text"
                      ? error.token.text
                      : null;

            if (!tokenSpan) {
                return null;
            }

            return {
                start: tokenSpan.start,
                end: tokenSpan.end,
            };
        }
        case "UndefinedNamespacePrefix":
            return {
                start: error.prefix.start,
                end: error.prefix.end,
            };
        case "UndefinedEntity":
            return {
                start: error.start,
                end: error.end,
            };
        case "InvalidOrUnclosedEntity":
            return {
                start: error.start,
                end: error.end,
            };
        default:
            return null;
    }
}

export function XmlEditor({ value, errors, onChange }: Props) {
    const lintExtension = useMemo(() => {
        return linter((view) => {
            const diagnostics: Diagnostic[] = [];

            for (const error of errors) {
                const span = toErrorSpan(error);
                if (!span) {
                    continue;
                }

                const { from, to } = clampSpan(
                    span.start,
                    span.end,
                    view.state.doc.length,
                );

                diagnostics.push({
                    from,
                    to,
                    severity: "error",
                    message: toDiagnosticMessage(error),
                    renderMessage: renderDiagnosticMessage(error),
                    source: "RelaxNG",
                });
            }

            return diagnostics;
        });
    }, [errors]);

    return (
        <CodeMirror
            value={value}
            height="calc(72vh - 82px)"
            extensions={[xml(), lintExtension]}
            onChange={onChange}
            basicSetup={{
                lineNumbers: true,
                foldGutter: true,
                highlightActiveLine: true,
            }}
        />
    );
}
