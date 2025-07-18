import { Link } from 'react-router-dom';

interface HeaderProps {
  onMenuClick: () => void;
}

export function Header({ onMenuClick }: HeaderProps) {
  return (
    <header className="bg-white shadow">
      <div className="container mx-auto px-4 py-4">
        <nav className="flex items-center justify-between">
          <button onClick={onMenuClick} className="p-2 hover:bg-gray-100 rounded-lg">
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
          <Link to="/" className="text-xl font-bold">
            EvoPro
          </Link>
          <div className="flex items-center space-x-4">
            <Link to="/agents" className="text-gray-600 hover:text-gray-900">
              Agents
            </Link>
            <Link to="/models" className="text-gray-600 hover:text-gray-900">
              Models
            </Link>
            <Link to="/settings" className="text-gray-600 hover:text-gray-900">
              Settings
            </Link>
          </div>
        </nav>
      </div>
    </header>
  );
} 