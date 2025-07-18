import { CheckSquare } from 'lucide-react'

export function TasksContent() {
    const tasks = [
      {
        id: 1,
        title: 'Review Design System',
        due: 'Today',
        priority: 'High',
        status: 'In Progress',
        description:
          'Review the new design system components and provide feedback.',
        assignee: 'You',
      },
      {
        id: 2,
        title: 'Update Documentation',
        due: 'Tomorrow',
        priority: 'Medium',
        status: 'To Do',
        description: 'Update the API documentation with the latest changes.',
        assignee: 'You',
      },
      {
        id: 3,
        title: 'Fix Navigation Bug',
        due: 'Today',
        priority: 'High',
        status: 'To Do',
        description:
          'The navigation menu disappears when clicking on certain items.',
        assignee: 'Sarah Miller',
      },
      {
        id: 4,
        title: 'Prepare Demo for Client',
        due: 'Tomorrow',
        priority: 'High',
        status: 'Not Started',
        description:
          'Create a demo presentation for the upcoming client meeting.',
        assignee: 'You',
      },
      {
        id: 5,
        title: 'Code Review PR #234',
        due: 'Today',
        priority: 'Medium',
        status: 'In Progress',
        description: 'Review the pull request for the new authentication system.',
        assignee: 'Alex Johnson',
      },
    ]
    return (
      <div className="space-y-4">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-semibold">My Tasks</h2>
          <button className="px-3 py-1 bg-primary text-primary-foreground rounded-md text-sm flex items-center gap-1">
            <CheckSquare size={16} />
            New Task
          </button>
        </div>
        <div className="grid grid-cols-1 gap-4">
          {tasks.map((task) => (
            <div
              key={task.id}
              className="border border-border rounded-lg p-4 hover:bg-accent/50 transition-colors"
            >
              <div className="flex justify-between items-start mb-2">
                <div className="flex items-start gap-3">
                  <div className="mt-1">
                    <input
                      type="checkbox"
                      className="rounded-sm border-primary h-4 w-4"
                    />
                  </div>
                  <div>
                    <h3 className="font-medium">{task.title}</h3>
                    <p className="text-sm text-muted-foreground mt-1">
                      {task.description}
                    </p>
                  </div>
                </div>
                <span
                  className={`text-xs px-2 py-1 rounded-full ${task.priority === 'High' ? 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200' : task.priority === 'Medium' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200' : 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'}`}
                >
                  {task.priority}
                </span>
              </div>
              <div className="flex justify-between mt-3 text-sm">
                <div className="flex items-center gap-4">
                  <span className="text-muted-foreground">
                    Due:{' '}
                    <span className="text-foreground font-medium">
                      {task.due}
                    </span>
                  </span>
                  <span className="text-muted-foreground">
                    Status:{' '}
                    <span className="text-foreground font-medium">
                      {task.status}
                    </span>
                  </span>
                  <span className="text-muted-foreground">
                    Assigned to:{' '}
                    <span className="text-foreground font-medium">
                      {task.assignee}
                    </span>
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    )
  }
  