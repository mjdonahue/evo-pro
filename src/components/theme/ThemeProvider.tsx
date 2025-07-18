import React, { useEffect, useState, createContext, useContext } from 'react';
type Theme = 'light' | 'dark';
type ThemeProviderProps = {
  children: React.ReactNode;
};
type ThemeProviderState = {
  theme: Theme;
  setTheme: (theme: Theme) => void;
};
const initialState: ThemeProviderState = {
  theme: 'light',
  setTheme: () => null
};
const ThemeProviderContext = createContext<ThemeProviderState>(initialState);
export function ThemeProvider({
  children,
  ...props
}: ThemeProviderProps) {
  const [theme, setTheme] = useState<Theme>('light');
  useEffect(() => {
    const root = window.document.documentElement;
    const savedTheme = localStorage.getItem('theme') as Theme;
    if (savedTheme) {
      setTheme(savedTheme);
      root.classList.remove('light', 'dark');
      root.classList.add(savedTheme);
    } else {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      setTheme(prefersDark ? 'dark' : 'light');
      root.classList.add(prefersDark ? 'dark' : 'light');
    }
  }, []);
  const value = {
    theme,
    setTheme: (theme: Theme) => {
      localStorage.setItem('theme', theme);
      const root = window.document.documentElement;
      root.classList.remove('light', 'dark');
      root.classList.add(theme);
      setTheme(theme);
    }
  };
  return <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>;
}
export const useTheme = () => {
  const context = useContext(ThemeProviderContext);
  if (context === undefined) throw new Error('useTheme must be used within a ThemeProvider');
  return context;
};