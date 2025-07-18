import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend } from 'k6/metrics';
import { baseConfig, createOptions } from './config.js';

// Define custom metrics
const errorRate = new Rate('error_rate');
const apiLatency = new Trend('api_latency');

// Export k6 options
export const options = createOptions('load', {
  // Add custom thresholds for this specific test
  'api_latency': [
    { threshold: 'p(95)<300', abortOnFail: false },
  ],
});

/**
 * Setup function that runs once at the beginning of the test
 */
export function setup() {
  // You can perform setup tasks here, like creating test data
  // or obtaining authentication tokens
  
  const loginRes = http.post(`${baseConfig.baseUrl}/api/auth/login`, JSON.stringify({
    username: 'testuser',
    password: 'testpassword',
  }), {
    headers: baseConfig.headers,
  });
  
  // Check if login was successful
  const success = check(loginRes, {
    'login successful': (r) => r.status === 200,
  });
  
  if (!success) {
    console.log('Login failed, proceeding with unauthenticated tests');
    return {};
  }
  
  // Extract token from response
  let token;
  try {
    token = JSON.parse(loginRes.body).token;
  } catch (e) {
    console.log('Failed to parse login response');
    return {};
  }
  
  return {
    token,
  };
}

/**
 * Default function that is executed for each virtual user
 */
export default function(data) {
  // Add authentication token to headers if available
  const headers = { ...baseConfig.headers };
  if (data.token) {
    headers['Authorization'] = `Bearer ${data.token}`;
  }
  
  // Main test flow
  group('API Endpoints', function() {
    // Test GET endpoints
    group('GET Requests', function() {
      // Get users
      let usersRes = http.get(`${baseConfig.baseUrl}/api/users`, { headers });
      
      // Check response and record metrics
      check(usersRes, {
        'users status is 200': (r) => r.status === 200,
        'users response has data': (r) => JSON.parse(r.body).length > 0,
      }) || errorRate.add(1);
      
      apiLatency.add(usersRes.timings.duration);
      
      // Get projects
      let projectsRes = http.get(`${baseConfig.baseUrl}/api/projects`, { headers });
      
      // Check response and record metrics
      check(projectsRes, {
        'projects status is 200': (r) => r.status === 200,
        'projects response has data': (r) => JSON.parse(r.body).length > 0,
      }) || errorRate.add(1);
      
      apiLatency.add(projectsRes.timings.duration);
      
      // Add a small delay between requests
      sleep(1);
    });
    
    // Test POST endpoints
    group('POST Requests', function() {
      // Create a new task
      const taskData = {
        title: `Test Task ${Date.now()}`,
        description: 'This is a test task created by k6',
        status: 'todo',
        priority: 'medium',
      };
      
      let createTaskRes = http.post(
        `${baseConfig.baseUrl}/api/tasks`,
        JSON.stringify(taskData),
        { headers }
      );
      
      // Check response and record metrics
      check(createTaskRes, {
        'create task status is 201': (r) => r.status === 201,
        'create task response has id': (r) => JSON.parse(r.body).id !== undefined,
      }) || errorRate.add(1);
      
      apiLatency.add(createTaskRes.timings.duration);
      
      // Add a small delay between requests
      sleep(1);
    });
    
    // Test PUT endpoints
    group('PUT Requests', function() {
      // First, get a task to update
      let tasksRes = http.get(`${baseConfig.baseUrl}/api/tasks`, { headers });
      
      let taskId;
      try {
        const tasks = JSON.parse(tasksRes.body);
        if (tasks.length > 0) {
          taskId = tasks[0].id;
        }
      } catch (e) {
        console.log('Failed to parse tasks response');
      }
      
      if (taskId) {
        // Update the task
        const updateData = {
          status: 'in_progress',
          priority: 'high',
        };
        
        let updateTaskRes = http.put(
          `${baseConfig.baseUrl}/api/tasks/${taskId}`,
          JSON.stringify(updateData),
          { headers }
        );
        
        // Check response and record metrics
        check(updateTaskRes, {
          'update task status is 200': (r) => r.status === 200,
          'update task response has updated fields': (r) => {
            const body = JSON.parse(r.body);
            return body.status === 'in_progress' && body.priority === 'high';
          },
        }) || errorRate.add(1);
        
        apiLatency.add(updateTaskRes.timings.duration);
      }
      
      // Add a small delay between requests
      sleep(1);
    });
    
    // Test DELETE endpoints
    group('DELETE Requests', function() {
      // First, get a task to delete
      let tasksRes = http.get(`${baseConfig.baseUrl}/api/tasks`, { headers });
      
      let taskId;
      try {
        const tasks = JSON.parse(tasksRes.body);
        if (tasks.length > 0) {
          // Get the last task to delete
          taskId = tasks[tasks.length - 1].id;
        }
      } catch (e) {
        console.log('Failed to parse tasks response');
      }
      
      if (taskId) {
        // Delete the task
        let deleteTaskRes = http.del(
          `${baseConfig.baseUrl}/api/tasks/${taskId}`,
          null,
          { headers }
        );
        
        // Check response and record metrics
        check(deleteTaskRes, {
          'delete task status is 204': (r) => r.status === 204,
        }) || errorRate.add(1);
        
        apiLatency.add(deleteTaskRes.timings.duration);
      }
      
      // Add a small delay between requests
      sleep(1);
    });
  });
  
  // Add a delay between iterations
  sleep(3);
}

/**
 * Teardown function that runs once at the end of the test
 */
export function teardown(data) {
  // You can perform cleanup tasks here, like deleting test data
  
  if (data.token) {
    // Logout
    http.post(`${baseConfig.baseUrl}/api/auth/logout`, null, {
      headers: {
        ...baseConfig.headers,
        'Authorization': `Bearer ${data.token}`,
      },
    });
  }
}