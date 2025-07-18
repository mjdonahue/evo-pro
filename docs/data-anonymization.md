# Data Anonymization Utilities

This document outlines the data anonymization utilities implemented in the evo-pro project to provide comprehensive privacy protection for user data.

## Overview

The data anonymization utilities provide a flexible and powerful framework for anonymizing various types of data, including:

1. **Structured Data**: User profiles, messages, and other structured data
2. **Unstructured Text**: Free-form text content that may contain sensitive information
3. **JSON Data**: Complex nested data structures
4. **Numeric Data**: Using differential privacy techniques
5. **Temporal Data**: Dates and timestamps

## Architecture

The anonymization system consists of the following components:

1. **Anonymization Strategies**: Different approaches to anonymizing data
2. **Anonymizer**: A configurable service for applying anonymization strategies
3. **Utility Functions**: Standalone functions for common anonymization tasks
4. **K-Anonymity Implementation**: For anonymizing datasets while preserving analytical utility

## Anonymization Strategies

The system supports multiple anonymization strategies that can be configured per field:

- **None**: No anonymization (for non-sensitive fields)
- **PartialMask**: Partial masking (e.g., "j*** d**")
- **CompleteMask**: Complete masking (e.g., "****")
- **Redaction**: Replacing with a redaction marker (e.g., "[REDACTED]")
- **Generalization**: Using less specific values (e.g., age ranges instead of exact age)
- **Pseudonymization**: Consistent replacement of values
- **DifferentialPrivacy**: Adding calibrated noise to numeric values

## Usage

### Basic Usage

```rust
// Create an anonymizer with default configuration
let anonymizer = Anonymizer::default();

// Anonymize different types of data
let anonymized_email = anonymizer.anonymize_email("john.doe@example.com");
let anonymized_phone = anonymizer.anonymize_phone("123-456-7890");
let anonymized_text = anonymizer.anonymize_text("My SSN is 123-45-6789");
```

### Custom Configuration

```rust
// Create a custom configuration
let config = AnonymizationConfig {
    default_strategy: AnonymizationStrategy::CompleteMask,
    field_strategies: {
        let mut map = HashMap::new();
        map.insert("email".to_string(), AnonymizationStrategy::PartialMask);
        map.insert("phone".to_string(), AnonymizationStrategy::Redaction);
        map
    },
    epsilon: 0.5, // Lower epsilon = more privacy for differential privacy
    preserve_format: true,
    ..Default::default()
};

// Create an anonymizer with custom configuration
let anonymizer = Anonymizer::new(config);
```

### Anonymizing JSON Data

```rust
// Anonymize a JSON object
let json_data = json!({
    "name": "John Doe",
    "email": "john.doe@example.com",
    "phone": "123-456-7890",
    "age": 35,
    "address": {
        "street": "123 Main St",
        "city": "Anytown",
        "zip": "12345"
    }
});

let anonymized_json = anonymizer.anonymize_json(&json_data);
```

### Applying K-Anonymity

```rust
// Apply k-anonymity to a dataset
let anonymized_dataset = anonymizer.apply_k_anonymity(
    &dataset,
    &["age", "zipcode", "gender"], // Quasi-identifiers
    5 // k value (minimum group size)
);
```

## Integration with Data Minimization Service

The anonymization utilities are integrated with the DataMinimizationService to provide comprehensive privacy protection:

```rust
// Create a data minimization service with custom anonymization config
let config = AnonymizationConfig {
    default_strategy: AnonymizationStrategy::PartialMask,
    // Additional configuration...
};
let service = DataMinimizationService::with_config(db, config);

// Anonymize user data
service.anonymize_user(&mut user).await?;

// Anonymize message content
service.anonymize_message(&mut message).await?;
```

## Best Practices

When using the anonymization utilities, follow these best practices:

1. **Configure Per Field**: Use different strategies for different fields based on sensitivity
2. **Consider Data Utility**: Balance privacy protection with maintaining data utility
3. **Test Thoroughly**: Verify that anonymization works as expected for your data
4. **Document Choices**: Document your anonymization strategy choices for compliance purposes
5. **Regular Review**: Periodically review and update anonymization strategies as needs change

## Privacy Considerations

The anonymization utilities help comply with privacy regulations by:

- **Minimizing Data**: Reducing the amount of sensitive data stored
- **Protecting Identities**: Making it difficult to identify individuals
- **Preserving Utility**: Maintaining the usefulness of data for legitimate purposes
- **Providing Options**: Offering different levels of anonymization based on context

## Future Enhancements

Planned enhancements to the anonymization utilities include:

1. Integration with natural language processing for better detection of sensitive information
2. Support for more advanced differential privacy techniques
3. Enhanced k-anonymity with l-diversity and t-closeness
4. Performance optimizations for large datasets