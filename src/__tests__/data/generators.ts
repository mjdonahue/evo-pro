/**
 * Test Data Generation Utilities
 * 
 * This module provides utilities for generating test data for different types of entities.
 * It uses a combination of predefined templates and random generation to create realistic test data.
 */

import { faker } from '@faker-js/faker';

/**
 * Interface for a test data generator
 */
export interface DataGenerator<T> {
  /**
   * Generates a single entity
   */
  one(overrides?: Partial<T>): T;

  /**
   * Generates multiple entities
   * @param count The number of entities to generate
   * @param overrides Optional overrides to apply to all entities
   */
  many(count: number, overrides?: Partial<T>): T[];

  /**
   * Generates an entity with specific attributes
   * @param attributes The attributes to set on the entity
   */
  with(attributes: Partial<T>): T;
}

/**
 * Creates a data generator for a specific entity type
 * @param template A function that returns a base entity
 * @returns A data generator for the entity type
 */
export function createGenerator<T>(template: () => T): DataGenerator<T> {
  return {
    one(overrides = {}) {
      return { ...template(), ...overrides };
    },

    many(count, overrides = {}) {
      return Array.from({ length: count }, () => ({ ...template(), ...overrides }));
    },

    with(attributes) {
      return { ...template(), ...attributes };
    },
  };
}

/**
 * User entity for testing
 */
export interface User {
  id: string;
  username: string;
  email: string;
  firstName: string;
  lastName: string;
  avatar?: string;
  role: 'admin' | 'user' | 'guest';
  createdAt: Date;
  updatedAt: Date;
}

/**
 * Task entity for testing
 */
export interface Task {
  id: string;
  title: string;
  description: string;
  status: 'todo' | 'in_progress' | 'done';
  priority: 'low' | 'medium' | 'high';
  dueDate?: Date;
  assigneeId?: string;
  createdAt: Date;
  updatedAt: Date;
  tags: string[];
}

/**
 * Project entity for testing
 */
export interface Project {
  id: string;
  name: string;
  description: string;
  status: 'active' | 'archived' | 'completed';
  startDate: Date;
  endDate?: Date;
  ownerId: string;
  memberIds: string[];
  createdAt: Date;
  updatedAt: Date;
}

/**
 * Comment entity for testing
 */
export interface Comment {
  id: string;
  content: string;
  authorId: string;
  entityId: string;
  entityType: 'task' | 'project';
  createdAt: Date;
  updatedAt: Date;
}

/**
 * User data generator
 */
export const userGenerator = createGenerator<User>(() => ({
  id: faker.string.uuid(),
  username: faker.internet.userName(),
  email: faker.internet.email(),
  firstName: faker.person.firstName(),
  lastName: faker.person.lastName(),
  avatar: faker.image.avatar(),
  role: faker.helpers.arrayElement(['admin', 'user', 'guest']),
  createdAt: faker.date.past(),
  updatedAt: faker.date.recent(),
}));

/**
 * Task data generator
 */
export const taskGenerator = createGenerator<Task>(() => ({
  id: faker.string.uuid(),
  title: faker.lorem.sentence({ min: 3, max: 8 }),
  description: faker.lorem.paragraph(),
  status: faker.helpers.arrayElement(['todo', 'in_progress', 'done']),
  priority: faker.helpers.arrayElement(['low', 'medium', 'high']),
  dueDate: faker.date.future(),
  assigneeId: faker.string.uuid(),
  createdAt: faker.date.past(),
  updatedAt: faker.date.recent(),
  tags: faker.helpers.arrayElements(['bug', 'feature', 'documentation', 'enhancement', 'design'], { min: 0, max: 3 }),
}));

/**
 * Project data generator
 */
export const projectGenerator = createGenerator<Project>(() => ({
  id: faker.string.uuid(),
  name: faker.commerce.productName(),
  description: faker.lorem.paragraph(),
  status: faker.helpers.arrayElement(['active', 'archived', 'completed']),
  startDate: faker.date.past(),
  endDate: faker.helpers.maybe(() => faker.date.future()),
  ownerId: faker.string.uuid(),
  memberIds: Array.from({ length: faker.number.int({ min: 1, max: 5 }) }, () => faker.string.uuid()),
  createdAt: faker.date.past(),
  updatedAt: faker.date.recent(),
}));

/**
 * Comment data generator
 */
export const commentGenerator = createGenerator<Comment>(() => ({
  id: faker.string.uuid(),
  content: faker.lorem.paragraph(),
  authorId: faker.string.uuid(),
  entityId: faker.string.uuid(),
  entityType: faker.helpers.arrayElement(['task', 'project']),
  createdAt: faker.date.past(),
  updatedAt: faker.date.recent(),
}));

/**
 * Generates related entities
 * @param count The number of related entities to generate
 * @returns An object containing related entities
 */
export function generateRelatedEntities(count = 1) {
  // Generate users
  const users = userGenerator.many(count * 2);
  
  // Generate projects with the generated users
  const projects = projectGenerator.many(count, {
    ownerId: users[0].id,
    memberIds: users.map(user => user.id),
  });
  
  // Generate tasks for each project
  const tasks = projects.flatMap(project => 
    taskGenerator.many(count * 3, {
      assigneeId: faker.helpers.arrayElement(users).id,
    })
  );
  
  // Generate comments for tasks and projects
  const taskComments = tasks.flatMap(task => 
    commentGenerator.many(faker.number.int({ min: 0, max: 3 }), {
      entityId: task.id,
      entityType: 'task',
      authorId: faker.helpers.arrayElement(users).id,
    })
  );
  
  const projectComments = projects.flatMap(project => 
    commentGenerator.many(faker.number.int({ min: 0, max: 2 }), {
      entityId: project.id,
      entityType: 'project',
      authorId: faker.helpers.arrayElement(users).id,
    })
  );
  
  const comments = [...taskComments, ...projectComments];
  
  return {
    users,
    projects,
    tasks,
    comments,
  };
}