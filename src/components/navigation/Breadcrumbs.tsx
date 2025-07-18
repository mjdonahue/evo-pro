import React, { Fragment } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { ChevronRight, Home } from 'lucide-react'
export function Breadcrumbs() {
  const location = useLocation()
  const pathnames = location.pathname.split('/').filter((x) => x)
  // If we're on the root or dashboard path, don't show breadcrumbs
  if (location.pathname === '/' || location.pathname === '/dashboard') {
    return null
  }
  return (
    <nav className="flex items-center text-sm text-muted-foreground mb-4">
      <Link to="/" className="hover:text-foreground flex items-center">
        <Home size={14} className="mr-1" />
        Home
      </Link>
      {pathnames.map((name, index) => {
        const routeTo = `/${pathnames.slice(0, index + 1).join('/')}`
        const isLast = index === pathnames.length - 1
        // Format the name to be more readable (capitalize first letter)
        const formattedName = name.charAt(0).toUpperCase() + name.slice(1)
        return (
          <Fragment key={name}>
            <ChevronRight size={14} className="mx-2" />
            {isLast ? (
              <span className="font-medium text-foreground">
                {formattedName}
              </span>
            ) : (
              <Link to={routeTo} className="hover:text-foreground">
                {formattedName}
              </Link>
            )}
          </Fragment>
        )
      })}
    </nav>
  )
}
