export function SettingsContent() {
    const categories = [
      {
        id: 'account',
        name: 'Account Settings',
        description: 'Manage your account information and preferences',
        settings: [
          {
            id: 'profile',
            name: 'Profile Information',
            description: 'Update your profile details and avatar',
            value: 'Edit',
          },
          {
            id: 'password',
            name: 'Password',
            description: 'Change your password',
            value: 'Change',
          },
          {
            id: 'email',
            name: 'Email Notifications',
            description: 'Configure when you receive email notifications',
            value: 'Daily Digest',
          },
        ],
      },
      {
        id: 'appearance',
        name: 'Appearance',
        description: 'Customize how the application looks',
        settings: [
          {
            id: 'theme',
            name: 'Theme',
            description: 'Choose between light and dark mode',
            value: 'System Default',
          },
          {
            id: 'density',
            name: 'Interface Density',
            description: 'Adjust the spacing in the user interface',
            value: 'Comfortable',
          },
        ],
      },
      {
        id: 'privacy',
        name: 'Privacy & Security',
        description: 'Manage your privacy and security settings',
        settings: [
          {
            id: 'sessions',
            name: 'Active Sessions',
            description: 'View and manage your active sessions',
            value: '2 Active',
          },
          {
            id: 'data',
            name: 'Data Usage',
            description: 'Control how your data is used and stored',
            value: 'Review',
          },
          {
            id: 'tfa',
            name: 'Two-Factor Authentication',
            description: 'Add an extra layer of security to your account',
            value: 'Disabled',
          },
        ],
      },
      {
        id: 'integrations',
        name: 'Integrations',
        description: 'Manage connections with other services',
        settings: [
          {
            id: 'github',
            name: 'GitHub',
            description: 'Connect your GitHub account',
            value: 'Connected',
          },
          {
            id: 'google',
            name: 'Google Workspace',
            description: 'Connect your Google account',
            value: 'Not Connected',
          },
          {
            id: 'slack',
            name: 'Slack',
            description: 'Connect your Slack workspace',
            value: 'Not Connected',
          },
        ],
      },
    ]
    return (
      <div className="max-w-4xl mx-auto">
        <h2 className="text-2xl font-bold mb-6">Settings</h2>
        <div className="space-y-8">
          {categories.map((category) => (
            <div key={category.id} className="border border-border rounded-lg">
              <div className="p-4 border-b border-border bg-accent/30">
                <h3 className="text-lg font-semibold">{category.name}</h3>
                <p className="text-sm text-muted-foreground mt-1">
                  {category.description}
                </p>
              </div>
              <div className="divide-y divide-border">
                {category.settings.map((setting) => (
                  <div
                    key={setting.id}
                    className="p-4 flex justify-between items-center"
                  >
                    <div>
                      <h4 className="font-medium">{setting.name}</h4>
                      <p className="text-sm text-muted-foreground mt-1">
                        {setting.description}
                      </p>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm text-muted-foreground">
                        {setting.value}
                      </span>
                      <button className="px-3 py-1.5 text-sm rounded-md border border-border hover:bg-accent/50 transition-colors">
                        Edit
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    )
  }
  