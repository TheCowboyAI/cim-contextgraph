#!/usr/bin/env bash

# Generate Test Documentation from Rust source
# This script extracts test documentation from the comprehensive test file
# and generates a markdown document

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_FILE="$PROJECT_ROOT/tests/comprehensive_contextgraph_tests.rs"
OUTPUT_FILE="$PROJECT_ROOT/docs/test-documentation.md"

echo "Generating test documentation from: $TEST_FILE"
echo "Output will be written to: $OUTPUT_FILE"

# Create docs directory if it doesn't exist
mkdir -p "$PROJECT_ROOT/docs"

# Start the markdown file
cat > "$OUTPUT_FILE" << 'EOF'
# ContextGraph Test Documentation

This document provides detailed documentation for each test in the comprehensive test suite.
Generated from rustdoc comments in `tests/comprehensive_contextgraph_tests.rs`.

## Test Overview

The comprehensive test suite validates all aspects of ContextGraph functionality through 20 user story-based tests.

EOF

# Function to extract test documentation
extract_test_doc() {
    local test_name="$1"
    local test_file="$2"

    # Use awk to extract the test function and its documentation
    awk -v test="$test_name" '
    BEGIN { in_test = 0; in_doc = 0; doc = ""; code = ""; }

    # Start capturing when we find the test
    $0 ~ "fn " test "\\(" {
        in_test = 1;
        # Print any accumulated doc
        if (doc != "") {
            print doc;
            doc = "";
        }
        code = $0 "\n";
        next;
    }

    # Capture documentation before test
    /^[[:space:]]*\/\/\// && !in_test {
        in_doc = 1;
        # Remove leading /// and spaces
        sub(/^[[:space:]]*\/\/\/[[:space:]]?/, "");
        doc = doc $0 "\n";
        next;
    }

    # Reset doc if we hit non-doc line
    !/^[[:space:]]*\/\// && !in_test {
        doc = "";
        in_doc = 0;
    }

    # Capture test code
    in_test {
        code = code $0 "\n";
        # Stop at closing brace at start of line
        if ($0 ~ /^}/) {
            in_test = 0;
            # Extract key information from code
            if (code ~ /Given:/) {
                print "\n**Test Structure:**";
                print "- **Given**: " extract_section(code, "Given:");
                print "- **When**: " extract_section(code, "When:");
                print "- **Then**: " extract_section(code, "Then:");
            }
            code = "";
        }
    }

    function extract_section(text, marker) {
        # Extract the comment after the marker
        match(text, marker " ([^\n]+)", arr);
        if (arr[1]) return arr[1];
        return "See test implementation";
    }
    ' "$test_file"
}

# Extract test names from the file
test_names=$(grep -E "^fn test_user_story_[0-9]+_" "$TEST_FILE" | sed 's/fn \(test_user_story_[^(]*\).*/\1/' | sort -V)

# Process each test
for test_name in $test_names; do
    # Extract test number and description
    test_num=$(echo "$test_name" | sed 's/test_user_story_\([0-9]*\)_.*/\1/')
    test_desc=$(echo "$test_name" | sed 's/test_user_story_[0-9]*_//' | sed 's/_/ /g')

    echo "Processing: $test_name"

    # Write test header
    cat >> "$OUTPUT_FILE" << EOF

## Test $test_num: ${test_desc^}

**Function**: \`$test_name\`

EOF

    # Extract and write test documentation
    extract_test_doc "$test_name" "$TEST_FILE" >> "$OUTPUT_FILE"

    # Add test categories based on test number
    case $test_num in
        1) echo -e "\n**Categories**: Core Functionality, Type System, Domain Modeling" >> "$OUTPUT_FILE" ;;
        2) echo -e "\n**Categories**: Event-Driven Architecture, Workflow Management" >> "$OUTPUT_FILE" ;;
        3) echo -e "\n**Categories**: Component System, Metadata Management, Policy" >> "$OUTPUT_FILE" ;;
        4) echo -e "\n**Categories**: Graph Structure, Relationships, Metadata" >> "$OUTPUT_FILE" ;;
        5) echo -e "\n**Categories**: Composition, Modularity, Subgraphs" >> "$OUTPUT_FILE" ;;
        6) echo -e "\n**Categories**: Graph Theory, Algorithms, Analysis" >> "$OUTPUT_FILE" ;;
        7) echo -e "\n**Categories**: Invariants, Validation, Business Rules" >> "$OUTPUT_FILE" ;;
        8) echo -e "\n**Categories**: Querying, Filtering, Search" >> "$OUTPUT_FILE" ;;
        9) echo -e "\n**Categories**: Network Analysis, Metrics, Analytics" >> "$OUTPUT_FILE" ;;
        10) echo -e "\n**Categories**: Category Theory, Mathematical Foundations" >> "$OUTPUT_FILE" ;;
        11) echo -e "\n**Categories**: Domain Integration, Entity Composition" >> "$OUTPUT_FILE" ;;
        12) echo -e "\n**Categories**: Workflow, State Machines, Process Modeling" >> "$OUTPUT_FILE" ;;
        13) echo -e "\n**Categories**: Knowledge Representation, Semantics, Concepts" >> "$OUTPUT_FILE" ;;
        14) echo -e "\n**Categories**: Event Sourcing, Event Flows, Visualization" >> "$OUTPUT_FILE" ;;
        15) echo -e "\n**Categories**: Domain Invariants, Consistency, Integrity" >> "$OUTPUT_FILE" ;;
        16) echo -e "\n**Categories**: Document Management, Approval Workflows" >> "$OUTPUT_FILE" ;;
        17) echo -e "\n**Categories**: AI Agents, Research Workflows, Automation" >> "$OUTPUT_FILE" ;;
        18) echo -e "\n**Categories**: Geographic Distribution, Deployment, Infrastructure" >> "$OUTPUT_FILE" ;;
        19) echo -e "\n**Categories**: Multi-Organization, Collaboration, B2B" >> "$OUTPUT_FILE" ;;
        20) echo -e "\n**Categories**: Security, Access Control, Policy Enforcement" >> "$OUTPUT_FILE" ;;
    esac
done

# Add summary section
cat >> "$OUTPUT_FILE" << 'EOF'

## Test Coverage Summary

### By Category

- **Core Functionality**: Tests 1, 5, 6
- **Domain Modeling**: Tests 1, 11, 15, 16-20
- **Event-Driven**: Tests 2, 14
- **Workflow Management**: Tests 2, 12, 16, 17
- **Metadata & Components**: Tests 3, 4, 8
- **Graph Theory**: Tests 6, 9, 10
- **Business Rules**: Tests 7, 15
- **Security & Policy**: Tests 3, 20
- **AI & Automation**: Test 17
- **Collaboration**: Tests 18, 19

### By Domain Entity

- **People**: All tests
- **Organizations**: Tests 1, 2, 3, 16-20
- **Agents**: Tests 1, 2, 17
- **Locations**: Tests 1, 18
- **Documents**: Tests 1, 3, 16, 17, 19
- **Policies**: Tests 1, 3, 16, 18, 20
- **Workflows**: Tests 2, 12, 16, 17

## Running the Tests

```bash
# Run all comprehensive tests
cargo test --test comprehensive_contextgraph_tests

# Run a specific test
cargo test --test comprehensive_contextgraph_tests test_user_story_1

# Run with output
cargo test --test comprehensive_contextgraph_tests -- --nocapture

# Run with specific log level
RUST_LOG=debug cargo test --test comprehensive_contextgraph_tests
```

## Test Maintenance

When adding new tests:
1. Follow the user story format
2. Include at least 3 domain entity types
3. Document with clear Given/When/Then structure
4. Add appropriate test categories
5. Update this documentation

EOF

echo "Test documentation generated successfully at: $OUTPUT_FILE"
