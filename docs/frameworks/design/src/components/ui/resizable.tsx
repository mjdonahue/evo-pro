import React, { forwardRef, Component } from 'react';
import { DragHandleDots2Icon } from '@radix-ui/react-icons';
import * as ResizablePrimitive from 'react-resizable-panels';
import { cn } from '@/lib/utils';
const ResizablePanelGroup = forwardRef<React.ElementRef<typeof ResizablePrimitive.PanelGroup>, ComponentPropsWithoutRef<typeof ResizablePrimitive.PanelGroup>>(({
  className,
  ...props
}, ref) => <ResizablePrimitive.PanelGroup ref={ref} className={cn('flex h-full w-full data-[panel-group-direction=vertical]:flex-col', className)} {...props} />);
ResizablePanelGroup.displayName = 'ResizablePanelGroup';
const ResizablePanel = forwardRef<React.ElementRef<typeof ResizablePrimitive.Panel>, ComponentPropsWithoutRef<typeof ResizablePrimitive.Panel>>(({
  className,
  ...props
}, ref) => <ResizablePrimitive.Panel ref={ref} className={cn('relative h-full', className)} {...props} />);
ResizablePanel.displayName = 'ResizablePanel';
const ResizableHandle = forwardRef<React.ElementRef<typeof ResizablePrimitive.PanelResizeHandle>, ComponentPropsWithoutRef<typeof ResizablePrimitive.PanelResizeHandle>>(({
  className,
  ...props
}, ref) => <ResizablePrimitive.PanelResizeHandle ref={ref} className={cn('relative flex w-px items-center justify-center bg-border after:absolute after:inset-y-0 after:left-1/2 after:w-1 after:-translate-x-1/2 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring focus-visible:ring-offset-1 data-[panel-group-direction=vertical]:h-px data-[panel-group-direction=vertical]:w-full data-[panel-group-direction=vertical]:after:left-0 data-[panel-group-direction=vertical]:after:h-1 data-[panel-group-direction=vertical]:after:w-full data-[panel-group-direction=vertical]:after:-translate-y-1/2 data-[panel-group-direction=vertical]:after:translate-x-0 [&[data-panel-group-direction=vertical]>div]:rotate-90', className)} {...props}>
    <div className="z-10 flex h-4 w-3 items-center justify-center rounded-sm border bg-border">
      <DragHandleDots2Icon className="h-2.5 w-2.5" />
    </div>
  </ResizablePrimitive.PanelResizeHandle>);
ResizableHandle.displayName = 'ResizableHandle';
export { ResizablePanelGroup, ResizablePanel, ResizableHandle };