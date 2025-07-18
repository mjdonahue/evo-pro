import { ApiClientError } from './client';

/**
 * Type validator function that validates data against a schema.
 * This function takes unknown data and returns it as the specified type T if validation passes.
 * If validation fails, it throws an ApiClientError.
 * 
 * @template T The type to validate against and return
 */
export type TypeValidator<T> = (data: unknown) => T;

/**
 * Creates a validator function for a specific type.
 * This function creates a TypeValidator that can be reused to validate multiple data instances
 * against the same schema.
 * 
 * @template T The type to validate against and return
 * @param schema The schema to validate against. Can be a primitive type constructor (String, Number, Boolean),
 *               an array with a single element representing the item schema, or an object with property schemas.
 * @returns A validator function that validates data against the schema
 * @throws {ApiClientError} If validation fails
 */
export function createValidator<T>(schema: any): TypeValidator<T> {
  return (data: unknown): T => {
    try {
      // Basic type checking
      if (data === null || data === undefined) {
        throw new Error('Data is null or undefined');
      }

      // For primitive types
      if (schema === String && typeof data !== 'string') {
        throw new Error(`Expected string, got ${typeof data}`);
      }
      if (schema === Number && typeof data !== 'number') {
        throw new Error(`Expected number, got ${typeof data}`);
      }
      if (schema === Boolean && typeof data !== 'boolean') {
        throw new Error(`Expected boolean, got ${typeof data}`);
      }

      // For arrays
      if (Array.isArray(schema)) {
        if (!Array.isArray(data)) {
          throw new Error(`Expected array, got ${typeof data}`);
        }

        const itemSchema = schema[0];
        const validator = createValidator(itemSchema);

        return data.map((item) => validator(item)) as unknown as T;
      }

      // For objects
      if (typeof schema === 'object' && schema !== null) {
        if (typeof data !== 'object' || data === null) {
          throw new Error(`Expected object, got ${typeof data}`);
        }

        const result: Record<string, any> = {};

        for (const key in schema) {
          if (Object.prototype.hasOwnProperty.call(schema, key)) {
            const propertySchema = schema[key];
            const propertyData = (data as Record<string, any>)[key];

            // Check if property exists
            if (propertyData === undefined) {
              // If the property is required in the schema
              if (!key.endsWith('?')) {
                throw new Error(`Missing required property: ${key}`);
              }
              continue;
            }

            // Validate property
            const propertyValidator = createValidator(propertySchema);
            result[key] = propertyValidator(propertyData);
          }
        }

        return result as T;
      }

      // If no validation rules matched, return the data as is
      return data as T;
    } catch (error) {
      throw new ApiClientError(
        'validation_error',
        error instanceof Error ? error.message : 'Validation error',
        { data }
      );
    }
  };
}

/**
 * Validates data against a schema.
 * This is a convenience function that creates a validator and immediately uses it.
 * 
 * @template T The type to validate against and return
 * @param data The data to validate
 * @param schema The schema to validate against. Can be a primitive type constructor (String, Number, Boolean),
 *               an array with a single element representing the item schema, or an object with property schemas.
 * @returns The validated data as type T
 * @throws {ApiClientError} If validation fails with code 'validation_error'
 * 
 * @example
 * // Validate a simple string
 * const validatedString = validateData<string>("hello", String);
 * 
 * @example
 * // Validate an object against an interface
 * interface User {
 *   id: string;
 *   name: string;
 *   age?: number;
 * }
 * 
 * const schema = {
 *   id: String,
 *   name: String,
 *   age: Number
 * };
 * 
 * const validatedUser = validateData<User>(userData, schema);
 */
export function validateData<T>(data: unknown, schema: any): T {
  const validator = createValidator<T>(schema);
  return validator(data);
}
