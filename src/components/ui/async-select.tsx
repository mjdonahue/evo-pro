import { useState, useEffect, useCallback } from 'react';
import { Check, ChevronsUpDown, Loader2 } from 'lucide-react';
import { useDebounce } from 'react-use';

import { cn } from '@/lib/utils';
import { Button } from './button';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from './command';
import { Popover, PopoverContent, PopoverTrigger } from './popover';

export interface Option {
  value: string;
  label: string;
  disabled?: boolean;
  description?: string;
  icon?: React.ReactNode;
}

export interface AsyncSelectProps<T> {
  /** Async function to fetch options */
  fetcher: (query?: string) => Promise<T[]>;
  /** Preload all data ahead of time */
  preload?: boolean;
  /** Function to filter options */
  filterFn?: (option: T, query: string) => boolean;
  /** Function to render each option */
  renderOption: (option: T) => React.ReactNode;
  /** Function to get the value from an option */
  getOptionValue: (option: T) => string;
  /** Function to get the display value for the selected option */
  getDisplayValue: (option: T) => React.ReactNode;
  /** Custom not found message */
  notFound?: React.ReactNode;
  /** Custom loading skeleton */
  loadingSkeleton?: React.ReactNode;
  /** Currently selected value */
  value: string;
  /** Callback when selection changes */
  onChange: (value: string) => void;
  /** Label for the select field */
  label: string;
  /** Placeholder text when no selection */
  placeholder?: string;
  /** Disable the entire select */
  disabled?: boolean;
  /** Custom width for the popover */
  width?: string | number;
  /** Custom class names */
  className?: string;
  /** Custom trigger button class names */
  triggerClassName?: string;
  /** Custom no results message */
  noResultsMessage?: string;
  /** Allow clearing the selection */
  clearable?: boolean;
  /** Default option to display when no selection */
  defaultOption?: T | null;
}

export function AsyncSelect<T>({
  fetcher,
  preload,
  filterFn,
  renderOption,
  getOptionValue,
  getDisplayValue,
  notFound,
  loadingSkeleton,
  label,
  placeholder = 'Select...',
  value,
  onChange,
  disabled = false,
  className,
  triggerClassName,
  noResultsMessage,
  clearable = true,
  defaultOption,
}: AsyncSelectProps<T>) {
  const [mounted, setMounted] = useState(false);
  const [open, setOpen] = useState(false);
  const [options, setOptions] = useState<T[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedValue, setSelectedValue] = useState(value);
  const [selectedOption, setSelectedOption] = useState<T | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [debouncedSearchTerm, setDebouncedSearchTerm] = useState('');
  const [originalOptions, setOriginalOptions] = useState<T[]>([]);
  const [initialFetchDone, setInitialFetchDone] = useState(false);
  useDebounce(() => setDebouncedSearchTerm(searchTerm), 250, [searchTerm]);

  // Handle initial mount and value setting
  useEffect(() => {
    if (!mounted) {
      setMounted(true);
      if (defaultOption) {
        setSelectedOption(defaultOption);
        setOptions([defaultOption]);
        setOriginalOptions([defaultOption]);
        setInitialFetchDone(true);
      }
    }
    setSelectedValue(value);
  }, [value, mounted, defaultOption]);

  // Initialize selectedOption when options are loaded and value exists
  useEffect(() => {
    if (value && options.length > 0) {
      const option = options.find((opt) => getOptionValue(opt) === value);
      if (option) {
        setSelectedOption(option);
      }
    }
  }, [value, options, getOptionValue]);

  useEffect(() => {
    const fetchOptions = async () => {
      // Skip if we're not searching and already have the default option
      if (!debouncedSearchTerm && defaultOption && !open) {
        return;
      }

      // Skip if we haven't mounted yet
      if (!mounted) {
        return;
      }

      try {
        setLoading(true);
        setError(null);

        // Only use value for initial fetch if we don't have a defaultOption
        const searchQuery =
          !initialFetchDone && value ? value : debouncedSearchTerm;

        const data = await fetcher(searchQuery);

        if (!initialFetchDone) {
          setInitialFetchDone(true);
        }

        setOriginalOptions(data);
        setOptions(data);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : 'Failed to fetch options'
        );
      } finally {
        setLoading(false);
      }
    };

    // Only fetch in these cases:
    // 1. Initial fetch with value (if no defaultOption)
    // 2. When search term changes and dropdown is open
    // 3. When dropdown opens
    if (
      (!initialFetchDone && value && !defaultOption) ||
      (open && debouncedSearchTerm !== undefined) ||
      (open && !initialFetchDone)
    ) {
      fetchOptions();
    } else if (preload) {
      if (debouncedSearchTerm) {
        setOptions(
          originalOptions.filter((option) =>
            filterFn ? filterFn(option, debouncedSearchTerm) : true
          )
        );
      } else {
        setOptions(originalOptions);
      }
    }
  }, [
    fetcher,
    debouncedSearchTerm,
    mounted,
    preload,
    filterFn,
    value,
    defaultOption,
    open,
    initialFetchDone,
  ]);

  const handleSelect = useCallback(
    (currentValue: string) => {
      const newValue =
        clearable && currentValue === selectedValue ? '' : currentValue;
      setSelectedValue(newValue);
      setSelectedOption(
        options.find((option) => getOptionValue(option) === newValue) || null
      );
      onChange(newValue);
      setOpen(false);
    },
    [selectedValue, onChange, clearable, options, getOptionValue]
  );

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className={cn(
            'w-full',
            'justify-between',
            disabled && 'opacity-50 cursor-not-allowed',
            triggerClassName
          )}
          disabled={disabled}
        >
          <div className="truncate flex-1 text-left">
            {selectedOption ? getDisplayValue(selectedOption) : placeholder}
          </div>
          <ChevronsUpDown className="opacity-50 ml-2 flex-shrink-0" size={10} />
        </Button>
      </PopoverTrigger>
      <PopoverContent className={cn('p-0', className)}>
        <Command shouldFilter={false}>
          <div className="relative border-b w-full">
            <CommandInput
              placeholder={`Search ${label.toLowerCase()}...`}
              value={searchTerm}
              onValueChange={(value: string) => {
                setSearchTerm(value);
              }}
            />
            {loading && options.length > 0 && (
              <div className="absolute right-2 top-1/2 transform -translate-y-1/2 flex items-center">
                <Loader2 className="h-4 w-4 animate-spin" />
              </div>
            )}
          </div>
          <CommandList>
            {error && (
              <div className="p-4 text-destructive text-center">{error}</div>
            )}
            {loading &&
              options.length === 0 &&
              (loadingSkeleton || <DefaultLoadingSkeleton />)}
            {!loading &&
              !error &&
              options.length === 0 &&
              (notFound || (
                <CommandEmpty>
                  {noResultsMessage ?? `No ${label.toLowerCase()} found.`}
                </CommandEmpty>
              ))}
            <CommandGroup>
              {options.map((option) => (
                <CommandItem
                  key={getOptionValue(option)}
                  value={getOptionValue(option)}
                  onSelect={handleSelect}
                >
                  {renderOption(option)}
                  <Check
                    className={cn(
                      'ml-auto h-3 w-3',
                      selectedValue === getOptionValue(option)
                        ? 'opacity-100'
                        : 'opacity-0'
                    )}
                  />
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

function DefaultLoadingSkeleton() {
  return (
    <CommandGroup>
      {[1, 2, 3].map((i) => (
        <CommandItem key={i} disabled>
          <div className="flex items-center gap-2 w-full">
            <div className="h-6 w-6 rounded-full animate-pulse bg-muted" />
            <div className="flex flex-col flex-1 gap-1">
              <div className="h-4 w-24 animate-pulse bg-muted rounded" />
              <div className="h-3 w-16 animate-pulse bg-muted rounded" />
            </div>
          </div>
        </CommandItem>
      ))}
    </CommandGroup>
  );
}
