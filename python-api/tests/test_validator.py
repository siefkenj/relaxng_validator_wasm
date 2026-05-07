import json
import pytest

import relaxng_validator as rv


SIMPLE_SCHEMA = json.dumps({"main.rnc": "start = element root { text }"})
CHOICE_SCHEMA = json.dumps(
    {"main.rnc": "start = element root { element foo { text } | element bar { text } }"}
)
ATTRS_SCHEMA = json.dumps(
    {
        "main.rnc": (
            "start = element book { "
            "  attribute isbn { text },"
            "  attribute year { text }?,"
            "  element title { text }"
            "}"
        )
    }
)


# ---------------------------------------------------------------------------
# validate() — one-shot function
# ---------------------------------------------------------------------------


class TestValidateFunction:
    def test_valid_document_returns_empty_list(self):
        errors = rv.validate(SIMPLE_SCHEMA, '<?xml version="1.0"?><root>hello</root>')
        assert errors == []

    def test_invalid_document_returns_errors(self):
        errors = rv.validate(
            SIMPLE_SCHEMA, '<?xml version="1.0"?><root><unexpected/></root>'
        )
        assert len(errors) > 0

    def test_wrong_element_lists_expected_elements(self):
        errors = rv.validate(
            CHOICE_SCHEMA, '<?xml version="1.0"?><root><baz/></root>'
        )
        not_allowed = [e for e in errors if e.type == "NotAllowed"]
        assert len(not_allowed) > 0
        first = not_allowed[0]
        assert "foo" in first.expected_elements
        assert "bar" in first.expected_elements

    def test_expected_elements_empty_for_text_content_model(self):
        errors = rv.validate(
            SIMPLE_SCHEMA, '<?xml version="1.0"?><root><unexpected/></root>'
        )
        not_allowed = [e for e in errors if e.type == "NotAllowed"]
        assert len(not_allowed) > 0
        assert not_allowed[0].expected_elements == []

    def test_bad_attribute_lists_expected_attributes(self):
        errors = rv.validate(
            ATTRS_SCHEMA,
            '<?xml version="1.0"?><book bad-attr="x"><title>Hi</title></book>',
        )
        not_allowed = [e for e in errors if e.type == "NotAllowed"]
        attr_err = next(
            (e for e in not_allowed if e.expected_attributes), None
        )
        assert attr_err is not None
        assert "isbn" in attr_err.expected_attributes

    def test_element_level_errors_have_empty_expected_attributes(self):
        errors = rv.validate(
            CHOICE_SCHEMA, '<?xml version="1.0"?><root><baz/></root>'
        )
        not_allowed = [e for e in errors if e.type == "NotAllowed"]
        assert len(not_allowed) > 0
        for err in not_allowed:
            assert err.expected_attributes == []

    def test_each_error_has_type_string(self):
        errors = rv.validate(
            SIMPLE_SCHEMA, '<?xml version="1.0"?><root><bad/></root>'
        )
        for err in errors:
            assert isinstance(err.type, str)

    def test_invalid_json_raises_runtime_error(self):
        with pytest.raises(RuntimeError):
            rv.validate("not json", '<?xml version="1.0"?><root/>')


# ---------------------------------------------------------------------------
# compile_validator() / Validator
# ---------------------------------------------------------------------------


class TestCompileValidator:
    @pytest.fixture
    def validator(self):
        return rv.compile_validator(SIMPLE_SCHEMA)

    def test_valid_document_returns_empty_list(self, validator):
        assert validator.validate('<?xml version="1.0"?><root>hello</root>') == []

    def test_invalid_document_returns_errors(self, validator):
        errors = validator.validate('<?xml version="1.0"?><root><child/></root>')
        assert len(errors) > 0

    def test_reusable_across_multiple_calls(self, validator):
        good = '<?xml version="1.0"?><root>text</root>'
        bad = '<?xml version="1.0"?><root><child/></root>'
        assert validator.validate(good) == []
        assert len(validator.validate(bad)) > 0
        assert validator.validate(good) == []

    def test_successive_invalid_documents_each_return_errors(self, validator):
        bad = '<?xml version="1.0"?><root><child/></root>'
        assert len(validator.validate(bad)) > 0
        assert len(validator.validate(bad)) > 0

    def test_invalid_json_raises_runtime_error(self):
        with pytest.raises(RuntimeError):
            rv.compile_validator("not json")


# ---------------------------------------------------------------------------
# First key as grammar entry point
# ---------------------------------------------------------------------------


class TestFirstKeyEntrypoint:
    def test_first_key_is_used_as_grammar_root(self):
        schema = json.dumps(
            {
                "main.rnc": "start = element doc { text }",
                "other.rnc": "start = element other { text }",
            }
        )
        v = rv.compile_validator(schema)
        assert v.validate('<?xml version="1.0"?><doc>hi</doc>') == []
        assert len(v.validate('<?xml version="1.0"?><other>hi</other>')) > 0


# ---------------------------------------------------------------------------
# VFS byte-array file content
# ---------------------------------------------------------------------------


class TestVfsByteArray:
    def test_schema_as_byte_array(self):
        schema_text = "start = element root { text }"
        byte_list = list(schema_text.encode("utf-8"))
        vfs_json = json.dumps({"main.rnc": byte_list})
        errors = rv.validate(vfs_json, '<?xml version="1.0"?><root>hello</root>')
        assert errors == []


# ---------------------------------------------------------------------------
# ValidationError attributes
# ---------------------------------------------------------------------------


class TestValidationErrorAttributes:
    def test_not_allowed_has_expected_fields(self):
        errors = rv.validate(
            CHOICE_SCHEMA, '<?xml version="1.0"?><root><baz/></root>'
        )
        err = next(e for e in errors if e.type == "NotAllowed")
        assert isinstance(err.expected_elements, list)
        assert isinstance(err.expected_attributes, list)
        assert isinstance(err.token, str)

    def test_repr_contains_type(self):
        errors = rv.validate(
            SIMPLE_SCHEMA, '<?xml version="1.0"?><root><bad/></root>'
        )
        assert len(errors) > 0
        assert "NotAllowed" in repr(errors[0])
