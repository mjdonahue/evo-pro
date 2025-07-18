import React from 'react';
import { MainLayout } from './components/layout/MainLayout';
import { ThemeProvider } from './components/theme/ThemeProvider';
export function App() {
  return <ThemeProvider children={undefined}>
      <div className="w-full h-screen bg-background text-foreground overflow-hidden">
        <MainLayout />
      </div>
    </ThemeProvider>;
}