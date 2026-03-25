import {
  SparkleIcon,
} from '@phosphor-icons/react';
import { cn } from '../lib/cn';

interface ModelProviderIconProps {
  providerId: string;
  theme?: 'light' | 'dark';
}

export function ModelProviderIcon({
  providerId,
  theme = 'light',
}: ModelProviderIconProps) {
  const suffix = theme === 'dark' ? '-dark' : '-light';
  const id = providerId.toLowerCase();
  const className = cn('size-icon-sm', 'flex-shrink-0');

  if (id.includes('anthropic') || id.includes('claude')) {
    return (
      <img
        src={`/agents/claude${suffix}.svg`}
        alt="Anthropic"
        className={className}
      />
    );
  }

  return <SparkleIcon className={className} />;
}
