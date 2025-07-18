import { describe, it, expect } from 'vitest';
import {
  userGenerator,
  taskGenerator,
  projectGenerator,
  commentGenerator,
  generateRelatedEntities,
  User,
  Task,
  Project,
  Comment,
} from './generators';

/**
 * Example tests for the test data generators
 * 
 * These tests demonstrate how to use the test data generators
 * to create test data for different scenarios.
 */
describe('Test Data Generators', () => {
  describe('User Generator', () => {
    it('generates a single user', () => {
      const user = userGenerator.one();
      
      // Verify that the user has all required properties
      expect(user).toHaveProperty('id');
      expect(user).toHaveProperty('username');
      expect(user).toHaveProperty('email');
      expect(user).toHaveProperty('firstName');
      expect(user).toHaveProperty('lastName');
      expect(user).toHaveProperty('role');
      expect(user).toHaveProperty('createdAt');
      expect(user).toHaveProperty('updatedAt');
      
      // Verify that the user's email is valid
      expect(user.email).toMatch(/^[^\s@]+@[^\s@]+\.[^\s@]+$/);
    });
    
    it('generates multiple users', () => {
      const users = userGenerator.many(5);
      
      // Verify that the correct number of users was generated
      expect(users).toHaveLength(5);
      
      // Verify that each user has a unique ID
      const ids = users.map(user => user.id);
      expect(new Set(ids).size).toBe(5);
    });
    
    it('generates a user with specific attributes', () => {
      const user = userGenerator.with({
        username: 'testuser',
        email: 'test@example.com',
        role: 'admin',
      });
      
      // Verify that the specified attributes were set
      expect(user.username).toBe('testuser');
      expect(user.email).toBe('test@example.com');
      expect(user.role).toBe('admin');
      
      // Verify that other attributes were still generated
      expect(user).toHaveProperty('id');
      expect(user).toHaveProperty('firstName');
      expect(user).toHaveProperty('lastName');
    });
  });
  
  describe('Task Generator', () => {
    it('generates a single task', () => {
      const task = taskGenerator.one();
      
      // Verify that the task has all required properties
      expect(task).toHaveProperty('id');
      expect(task).toHaveProperty('title');
      expect(task).toHaveProperty('description');
      expect(task).toHaveProperty('status');
      expect(task).toHaveProperty('priority');
      expect(task).toHaveProperty('createdAt');
      expect(task).toHaveProperty('updatedAt');
      expect(task).toHaveProperty('tags');
      
      // Verify that the task's status is valid
      expect(['todo', 'in_progress', 'done']).toContain(task.status);
    });
    
    it('generates tasks with specific attributes', () => {
      const tasks = taskGenerator.many(3, {
        status: 'todo',
        priority: 'high',
      });
      
      // Verify that all tasks have the specified attributes
      tasks.forEach(task => {
        expect(task.status).toBe('todo');
        expect(task.priority).toBe('high');
      });
    });
  });
  
  describe('Project Generator', () => {
    it('generates a project with specific attributes', () => {
      const project = projectGenerator.with({
        name: 'Test Project',
        status: 'active',
      });
      
      // Verify that the specified attributes were set
      expect(project.name).toBe('Test Project');
      expect(project.status).toBe('active');
      
      // Verify that other attributes were still generated
      expect(project).toHaveProperty('id');
      expect(project).toHaveProperty('description');
      expect(project).toHaveProperty('startDate');
      expect(project).toHaveProperty('ownerId');
      expect(project).toHaveProperty('memberIds');
      expect(project.memberIds).toBeInstanceOf(Array);
    });
  });
  
  describe('Related Entities Generator', () => {
    it('generates related entities', () => {
      const { users, projects, tasks, comments } = generateRelatedEntities(2);
      
      // Verify that the correct number of entities was generated
      expect(users).toHaveLength(4); // 2 * 2 users
      expect(projects).toHaveLength(2); // 2 projects
      expect(tasks).toHaveLength(12); // 2 projects * 2 * 3 tasks
      
      // Verify that the relationships are correct
      expect(projects[0].ownerId).toBe(users[0].id);
      expect(projects[0].memberIds).toContain(users[0].id);
      
      // Verify that all comments have valid entity types
      comments.forEach(comment => {
        expect(['task', 'project']).toContain(comment.entityType);
        expect(users.map(u => u.id)).toContain(comment.authorId);
      });
    });
    
    it('can be used to set up complex test scenarios', () => {
      // Generate a set of related entities
      const { users, projects, tasks } = generateRelatedEntities(1);
      
      // Use the generated entities to set up a test scenario
      const admin = users[0];
      const regularUser = users[1];
      const project = projects[0];
      const adminTask = taskGenerator.with({
        assigneeId: admin.id,
        title: 'Admin Task',
      });
      const userTask = taskGenerator.with({
        assigneeId: regularUser.id,
        title: 'User Task',
      });
      
      // Verify the test scenario
      expect(admin.role).toBeDefined();
      expect(regularUser.role).toBeDefined();
      expect(project.memberIds).toContain(admin.id);
      expect(project.memberIds).toContain(regularUser.id);
      expect(adminTask.assigneeId).toBe(admin.id);
      expect(userTask.assigneeId).toBe(regularUser.id);
      
      // This is where you would use these entities in your actual test
      // For example, testing authorization rules:
      const canEditTask = (user: User, task: Task): boolean => {
        return user.role === 'admin' || task.assigneeId === user.id;
      };
      
      expect(canEditTask(admin, adminTask)).toBe(true);
      expect(canEditTask(admin, userTask)).toBe(true); // Admin can edit any task
      expect(canEditTask(regularUser, userTask)).toBe(true); // User can edit their own task
      expect(canEditTask(regularUser, adminTask)).toBe(false); // User cannot edit admin's task
    });
  });
});