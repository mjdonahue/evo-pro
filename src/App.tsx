import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { ThemeProvider } from './components/theme/ThemeProvider'
import { ErrorBoundary } from './components/ErrorBoundary'
import { globalErrorHandler } from './utils/errorHandler'
import React from 'react'
import { lazyLoad } from './utils/lazyLoad'

// Lazy load route components
const MainLayout = lazyLoad(() => import('./components/layout/MainLayout').then(module => ({ default: module.MainLayout })))
const ChatDemo = lazyLoad(() => import('./components/ChatDemo').then(module => ({ default: module.ChatDemo })))

export function App() {
      // Initialize global error handler
  React.useEffect(() => {
    // Log app startup (commented out to avoid loops during development)
    // globalErrorHandler.logError('Application started', { 
    //   version: '1.0.0',
    //   timestamp: new Date().toISOString()
    // });
  }, []);

  return (
    <ThemeProvider>
      <ErrorBoundary onError={(error, errorInfo) => globalErrorHandler.logError(error, { errorInfo })}> 
      <Router>
        <div className="w-full h-screen bg-background text-foreground overflow-hidden">
          <Routes>
            <Route path="/chat" element={<ChatDemo />} />
            <Route path="/*" element={<MainLayout />} />
          </Routes>
        </div>
      </Router>
      </ErrorBoundary>
    </ThemeProvider>
  )
}
