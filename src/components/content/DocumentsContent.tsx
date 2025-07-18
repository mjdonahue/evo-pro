import { Folder, FileText } from 'lucide-react'

export function DocumentsContent() {
    const documents = [
      {
        id: 1,
        name: 'Project Documentation',
        type: 'folder',
        items: 5,
        modified: '2h ago',
        owner: 'You',
      },
      {
        id: 2,
        name: 'Meeting Notes.md',
        type: 'file',
        modified: '3h ago',
        owner: 'Sarah Miller',
        size: '24 KB',
      },
      {
        id: 3,
        name: 'Design Assets',
        type: 'folder',
        items: 12,
        modified: 'Yesterday',
        owner: 'Design Team',
      },
      {
        id: 4,
        name: 'API Specification.md',
        type: 'file',
        modified: 'Yesterday',
        owner: 'Alex Johnson',
        size: '156 KB',
      },
      {
        id: 5,
        name: 'Quarterly Report.pdf',
        type: 'file',
        modified: '2d ago',
        owner: 'You',
        size: '2.4 MB',
      },
      {
        id: 6,
        name: 'User Research',
        type: 'folder',
        items: 8,
        modified: '3d ago',
        owner: 'Research Team',
      },
    ]
    return (
      <div>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-xl font-semibold">Documents</h2>
          <div className="flex gap-2">
            <button className="px-3 py-1 bg-accent text-accent-foreground rounded-md text-sm flex items-center gap-1">
              <Folder size={16} />
              New Folder
            </button>
            <button className="px-3 py-1 bg-primary text-primary-foreground rounded-md text-sm flex items-center gap-1">
              <FileText size={16} />
              Upload
            </button>
          </div>
        </div>
        <div className="border rounded-lg overflow-hidden">
          <div className="grid grid-cols-12 bg-accent/50 p-3 text-sm font-medium text-muted-foreground">
            <div className="col-span-6">Name</div>
            <div className="col-span-2">Owner</div>
            <div className="col-span-2">Modified</div>
            <div className="col-span-2">Size</div>
          </div>
          {documents.map((doc) => (
            <div
              key={doc.id}
              className="grid grid-cols-12 p-3 hover:bg-accent/30 transition-colors border-t border-border"
            >
              <div className="col-span-6 flex items-center gap-2">
                {doc.type === 'folder' ? (
                  <Folder size={16} className="text-primary" />
                ) : (
                  <FileText size={16} className="text-primary" />
                )}
                <span className="font-medium">{doc.name}</span>
              </div>
              <div className="col-span-2 text-sm text-muted-foreground">
                {doc.owner}
              </div>
              <div className="col-span-2 text-sm text-muted-foreground">
                {doc.modified}
              </div>
              <div className="col-span-2 text-sm text-muted-foreground">
                {doc.type === 'folder' ? `${doc.items} items` : doc.size}
              </div>
            </div>
          ))}
        </div>
      </div>
    )
  }
  