import { Calendar, Users } from 'lucide-react'

export function CalendarContent() {
    const today = new Date()
    const formattedDate = today.toLocaleDateString('en-US', {
      weekday: 'long',
      month: 'long',
      day: 'numeric',
    })
    const events = [
      {
        id: 1,
        title: 'Team Standup',
        time: '09:00 AM',
        duration: '30m',
        participants: 5,
        location: 'Meeting Room A',
      },
      {
        id: 2,
        title: 'Design Review',
        time: '10:30 AM',
        duration: '1h',
        participants: 4,
        location: 'Zoom Call',
      },
      {
        id: 3,
        title: 'Lunch Break',
        time: '12:00 PM',
        duration: '1h',
        participants: 0,
        location: 'Cafeteria',
      },
      {
        id: 4,
        title: 'Product Planning',
        time: '1:30 PM',
        duration: '1h 30m',
        participants: 6,
        location: 'Conference Room',
      },
      {
        id: 5,
        title: 'Client Meeting',
        time: '3:00 PM',
        duration: '45m',
        participants: 3,
        location: 'Zoom Call',
      },
    ]
    return (
      <div>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-xl font-semibold">{formattedDate}</h2>
          <button className="px-3 py-1 bg-primary text-primary-foreground rounded-md text-sm flex items-center gap-1">
            <Calendar size={16} />
            New Event
          </button>
        </div>
        <div className="space-y-4">
          {events.map((event) => (
            <div
              key={event.id}
              className="border-l-4 border-primary pl-4 py-2 hover:bg-accent/50 transition-colors rounded-r-md"
            >
              <div className="flex justify-between items-start">
                <div>
                  <h3 className="font-medium">{event.title}</h3>
                  <div className="flex items-center gap-2 mt-1 text-sm text-muted-foreground">
                    <span className="text-primary">{event.time}</span>
                    <span>•</span>
                    <span>{event.duration}</span>
                    <span>•</span>
                    <span>{event.location}</span>
                  </div>
                </div>
                {event.participants > 0 && (
                  <div className="flex items-center gap-1 text-sm text-muted-foreground">
                    <Users size={14} />
                    <span>{event.participants}</span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    )
  }
  