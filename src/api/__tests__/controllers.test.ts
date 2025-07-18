import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { controllers, ControllerError, ConversationController } from '../controllers';
import { ipc_invoke } from '../ipc';

// Mock the ipc_invoke function
vi.mock('../ipc', () => ({
  ipc_invoke: vi.fn(),
}));

describe('Controllers', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('BaseController', () => {
    it('should get an entity by ID', async () => {
      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: { id: '123', name: 'Test Entity' },
      });

      // Call the get method on the conversations controller
      const result = await controllers.conversations.get('123');

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('get_conversation', { id: '123' });

      // Verify the result
      expect(result).toEqual({ id: '123', name: 'Test Entity' });
    });

    it('should handle errors when getting an entity', async () => {
      // Mock the ipc_invoke function to return an error response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: false,
        error: 'Entity not found',
      });

      // Call the get method on the conversations controller and expect it to throw
      await expect(controllers.conversations.get('123')).rejects.toThrow(ControllerError);

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('get_conversation', { id: '123' });
    });

    it('should create an entity', async () => {
      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: { id: '123', title: 'Test Conversation' },
      });

      // Call the create method on the conversations controller
      const result = await controllers.conversations.create({
        title: 'Test Conversation',
        type: 'Group',
      });

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('create_conversation', {
        input: {
          title: 'Test Conversation',
          type: 'Group',
        },
      });

      // Verify the result
      expect(result).toEqual({ id: '123', title: 'Test Conversation' });
    });

    it('should update an entity', async () => {
      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: { id: '123', title: 'Updated Conversation' },
      });

      // Call the update method on the conversations controller
      const result = await controllers.conversations.update('123', {
        id: '123',
        title: 'Updated Conversation',
        type: 'Group',
      });

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('update_conversation', {
        id: '123',
        data: {
          id: '123',
          title: 'Updated Conversation',
          type: 'Group',
        },
      });

      // Verify the result
      expect(result).toEqual({ id: '123', title: 'Updated Conversation' });
    });

    it('should delete an entity', async () => {
      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: undefined,
      });

      // Call the delete method on the conversations controller
      await controllers.conversations.delete('123');

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('delete_conversation', { id: '123' });
    });

    it('should list entities', async () => {
      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: {
          items: [
            { id: '123', title: 'Conversation 1' },
            { id: '456', title: 'Conversation 2' },
          ],
          total: 2,
        },
      });

      // Call the list method on the conversations controller
      const result = await controllers.conversations.list({ status: 'Active' });

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('list_conversations', {
        filter: { status: 'Active' },
      });

      // Verify the result
      expect(result).toEqual({
        items: [
          { id: '123', title: 'Conversation 1' },
          { id: '456', title: 'Conversation 2' },
        ],
        total: 2,
      });
    });
  });

  describe('ConversationController', () => {
    it('should get participants of a conversation', async () => {
      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: [
          { id: '123', display_name: 'User 1' },
          { id: '456', display_name: 'User 2' },
        ],
      });

      // Call the getParticipants method on the conversations controller
      const result = await controllers.conversations.getParticipants('123');

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('get_conversation_participants', {
        conversation_id: '123',
      });

      // Verify the result
      expect(result).toEqual([
        { id: '123', display_name: 'User 1' },
        { id: '456', display_name: 'User 2' },
      ]);
    });

    it('should add a participant to a conversation', async () => {
      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: undefined,
      });

      // Call the addParticipant method on the conversations controller
      await controllers.conversations.addParticipant('123', '456');

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('add_conversation_participant', {
        conversation_id: '123',
        user_id: '456',
      });
    });
  });

  describe('MessageController', () => {
    it('should mark a message as read', async () => {
      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: undefined,
      });

      // Call the markAsRead method on the messages controller
      await controllers.messages.markAsRead('123');

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('mark_message_read', { id: '123' });
    });
  });

  describe('TaskController', () => {
    it('should update the status of a task', async () => {
      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: undefined,
      });

      // Call the updateStatus method on the tasks controller
      await controllers.tasks.updateStatus({ id: '123', status: 'Completed' });

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('update_task_status', {
        id: '123',
        status: 'Completed',
      });
    });
  });

  describe('Controllers', () => {
    it('should check API health', async () => {
      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: undefined,
      });

      // Call the healthCheck method on the controllers
      const result = await controllers.healthCheck();

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('health_check');

      // Verify the result
      expect(result).toBe(true);
    });

    it('should handle errors when checking API health', async () => {
      // Mock the ipc_invoke function to throw an error
      vi.mocked(ipc_invoke).mockRejectedValueOnce(new Error('Connection error'));

      // Call the healthCheck method on the controllers
      const result = await controllers.healthCheck();

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('health_check');

      // Verify the result
      expect(result).toBe(false);
    });

    it('should get API version', async () => {
      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: '1.0.0',
      });

      // Call the getVersion method on the controllers
      const result = await controllers.getVersion();

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('get_version');

      // Verify the result
      expect(result).toBe('1.0.0');
    });
  });

  describe('Backward Compatibility', () => {
    it('should support the old Controller class', async () => {
      // Import the old Controller class
      const { Controller } = await import('../controllers');

      // Create a new controller
      const controller = new Controller<any, any, any>('test');

      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: { id: '123', name: 'Test Entity' },
      });

      // Call the get method on the controller
      const result = await controller.get('123');

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('get_test', { id: '123' });

      // Verify the result
      expect(result).toEqual({ id: '123', name: 'Test Entity' });
    });

    it('should support the old ChatController class', async () => {
      // Import the old ChatController class
      const { ChatController } = await import('../controllers');

      // Create a new chat controller
      const chatController = new ChatController();

      // Mock the ipc_invoke function to return a successful response
      vi.mocked(ipc_invoke).mockResolvedValueOnce({
        success: true,
        data: [{ id: '123', name: 'Test Chat' }],
      });

      // Call the list method on the chat controller
      const result = await chatController.list({ page: 1 });

      // Verify that ipc_invoke was called with the correct arguments
      expect(ipc_invoke).toHaveBeenCalledWith('list_chats', { page: { page: 1 } });

      // Verify the result
      expect(result).toEqual([{ id: '123', name: 'Test Chat' }]);
    });
  });
});