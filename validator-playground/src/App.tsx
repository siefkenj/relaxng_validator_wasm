import { useEffect, useMemo, useRef, useState } from "react";
import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import { releaseProxy, wrap } from "comlink";
import type { Remote } from "comlink";
import type {
    ValidationResponse,
    ValidatorWorkerApi,
} from "./validatorWorkerApi.ts";
import { XmlEditor } from "./XmlEditor.tsx";
import "./App.css";

type SchemaPreset = "rnc" | "rng" | "dev-plus-rnc";

type SchemaAssets = {
    pretextRnc: string;
    pretextRng: string;
    pretextDevRnc: string;
    testGoodXml: string;
};

const SCHEMA_LABELS: Record<SchemaPreset, string> = {
    rnc: "pretext.rnc",
    rng: "pretext.rng",
    "dev-plus-rnc": "pretext-dev.rnc + pretext.rnc",
};

const DEFAULT_XML = `<article xml:id="demo-article" xmlns="http://pretextbook.org/2020/pretext">
  <title>Minimal Demo</title>
  <p>This sample starts with test-good.xml content, then you can edit and revalidate.</p>
</article>
`;

const AUTO_VALIDATE_DEBOUNCE_MS = 300;

function App() {
    const [schemaPreset, setSchemaPreset] = useState<SchemaPreset>("rnc");
    const [schemaAssets, setSchemaAssets] = useState<SchemaAssets | null>(null);
    const [xmlText, setXmlText] = useState<string>(DEFAULT_XML);
    const [errors, setErrors] = useState<ValidationResponse["errors"]>([]);
    const [lastValidationMessage, setLastValidationMessage] =
        useState<string>("Not yet validated");
    const [isManualValidationRunning, setIsManualValidationRunning] =
        useState(false);

    const validatorRef = useRef<Remote<ValidatorWorkerApi> | null>(null);
    const validationRunIdRef = useRef(0);

    useEffect(() => {
        const worker = new Worker(
            new URL("./validator.worker.ts", import.meta.url),
            {
                type: "module",
            },
        );
        const validator = wrap<ValidatorWorkerApi>(worker);
        validatorRef.current = validator;

        return () => {
            validatorRef.current = null;
            validator[releaseProxy]();
            worker.terminate();
        };
    }, []);

    useEffect(() => {
        const loadAssets = async () => {
            const assetUrl = (name: string) =>
                `${import.meta.env.BASE_URL}assets/${name}`;

            const [pretextRnc, pretextRng, pretextDevRnc, testGoodXml] =
                await Promise.all([
                    fetch(assetUrl("pretext.rnc")).then((r) => r.text()),
                    fetch(assetUrl("pretext.rng")).then((r) => r.text()),
                    fetch(assetUrl("pretext-dev.rnc")).then((r) => r.text()),
                    fetch(assetUrl("test-good.xml")).then((r) => r.text()),
                ]);

            setSchemaAssets({
                pretextRnc,
                pretextRng,
                pretextDevRnc,
                testGoodXml,
            });
            setXmlText(testGoodXml);
            setLastValidationMessage(
                "Assets loaded. Validating changes automatically.",
            );
        };

        loadAssets().catch((e) => {
            const message = e instanceof Error ? e.message : String(e);
            setLastValidationMessage(
                `Failed to load symlinked test assets: ${message}`,
            );
        });
    }, []);

    const vfsJson = useMemo(() => {
        if (!schemaAssets) {
            return "";
        }

        if (schemaPreset === "rng") {
            return JSON.stringify({ "pretext.rng": schemaAssets.pretextRng });
        }

        if (schemaPreset === "dev-plus-rnc") {
            return JSON.stringify({
                "pretext-dev.rnc": schemaAssets.pretextDevRnc,
                "pretext.rnc": schemaAssets.pretextRnc,
            });
        }

        return JSON.stringify({ "pretext.rnc": schemaAssets.pretextRnc });
    }, [schemaAssets, schemaPreset]);

    const runValidation = async (isManualTrigger = false) => {
        if (!schemaAssets || !vfsJson) {
            setLastValidationMessage("Schema assets are not loaded yet.");
            return;
        }

        const validator = validatorRef.current;
        if (!validator) {
            setLastValidationMessage("Validation worker is not ready yet.");
            return;
        }

        const runId = ++validationRunIdRef.current;

        if (isManualTrigger) {
            setIsManualValidationRunning(true);
        }

        try {
            const result = await validator.validate(vfsJson, xmlText);

            if (runId !== validationRunIdRef.current) {
                return;
            }

            setErrors(result.errors);
            if (result.errors.length === 0) {
                setLastValidationMessage("Valid XML. No schema errors found.");
            } else {
                setLastValidationMessage(
                    `Found ${result.errors.length} schema error(s).`,
                );
            }
        } catch (e) {
            if (runId !== validationRunIdRef.current) {
                return;
            }

            const message = e instanceof Error ? e.message : String(e);
            setErrors([]);
            setLastValidationMessage(`Validation failed to run: ${message}`);
        } finally {
            if (isManualTrigger) {
                setIsManualValidationRunning(false);
            }
        }
    };

    useEffect(() => {
        if (!schemaAssets) {
            return;
        }

        const timeoutId = window.setTimeout(() => {
            void runValidation(false);
        }, AUTO_VALIDATE_DEBOUNCE_MS);

        return () => {
            window.clearTimeout(timeoutId);
        };
    }, [schemaAssets, schemaPreset, vfsJson, xmlText]);

    return (
        <main className="page">
            <header className="topbar">
                <div>
                    <h1>PreTeXt RelaxNG Playground</h1>
                    <p>
                        Live XML editing on the left, schema validation
                        diagnostics on the right.
                    </p>
                </div>
                <div className="controls">
                    <label htmlFor="schema-preset">Schema preset</label>
                    <select
                        id="schema-preset"
                        value={schemaPreset}
                        onChange={(e) =>
                            setSchemaPreset(e.target.value as SchemaPreset)
                        }
                    >
                        <option value="rnc">{SCHEMA_LABELS.rnc}</option>
                        <option value="rng">{SCHEMA_LABELS.rng}</option>
                        <option value="dev-plus-rnc">
                            {SCHEMA_LABELS["dev-plus-rnc"]}
                        </option>
                    </select>
                    <button
                        type="button"
                        onClick={() => {
                            void runValidation(true);
                        }}
                        disabled={isManualValidationRunning}
                    >
                        {isManualValidationRunning
                            ? "Validating..."
                            : "Validate"}
                    </button>
                </div>
            </header>

            <PanelGroup
                direction="horizontal"
                className="workspace-panel-group"
            >
                <Panel
                    defaultSize={55}
                    minSize={25}
                    className="workspace-panel"
                >
                    <section className="pane pane-editor">
                        <h2>XML Editor</h2>
                        <p className="hint">
                            Loaded from symlinked tests/test-good.xml; edit and
                            validate.
                        </p>
                        <XmlEditor
                            value={xmlText}
                            errors={errors}
                            onChange={setXmlText}
                        />
                    </section>
                </Panel>

                <PanelResizeHandle className="splitter" />

                <Panel
                    defaultSize={45}
                    minSize={25}
                    className="workspace-panel"
                >
                    <section className="pane pane-results">
                        <h2>Validation Result</h2>
                        <p className="status">{lastValidationMessage}</p>

                        {errors.length === 0 ? (
                            <div className="ok">No errors to show.</div>
                        ) : (
                            <ol className="errors">
                                {errors.map((err, idx) => (
                                    <li key={`${err.type ?? "unknown"}-${idx}`}>
                                        <div className="error-type">
                                            {err.type ?? "UnknownError"}
                                        </div>
                                        <pre>
                                            {JSON.stringify(err, null, 2)}
                                        </pre>
                                    </li>
                                ))}
                            </ol>
                        )}
                    </section>
                </Panel>
            </PanelGroup>
        </main>
    );
}

export default App;
