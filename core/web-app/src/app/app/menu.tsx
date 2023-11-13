'use client';
import classNames from 'classnames';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

function getMenuClassNames(currentPathname: string, targetPath?: string) {
  return classNames(
    !targetPath || currentPathname.indexOf(targetPath) < 0
      ? 'text-gray-600 pr-4 pl-4'
      : 'text-gray-1000 pr-4 pl-4'
  );
}

export default function Menu() {
  const pathname = usePathname();
  return (
    <ul className="flex h-full flex-row items-center pt-1 lg:flex-row">
      <li
        className={classNames(
          'flex-column flex h-full items-center lg:flex-row',
          pathname === '/app/autoscaling-history'
            ? 'border-b-4 border-purple-500'
            : 'border-b-4 border-white'
        )}
      >
        <Link
          className={classNames(
            getMenuClassNames(pathname, '/app/autoscaling-history'),
            'whitespace-nowrap'
          )}
          href="/app/autoscaling-history"
        >
          Autoscaling History
        </Link>
      </li>
      <li
        className={classNames(
          'flex-column flex h-full items-center lg:flex-row',
          pathname === '/app/planning'
            ? 'border-b-4 border-purple-500'
            : 'border-b-4 border-white'
        )}
      >
        <Link
          className={classNames(
            getMenuClassNames(pathname, '/app/planning'),
            'whitespace-nowrap'
          )}
          href="/app/planning"
        >
          Planning
        </Link>
      </li>
      <li
        className={classNames(
          'flex-column flex h-full items-center lg:flex-row',
          pathname === '/app/metrics'
            ? 'border-b-4 border-purple-500'
            : 'border-b-4 border-white'
        )}
      >
        <Link
          className={classNames(
            getMenuClassNames(pathname, '/app/metrics'),
            'whitespace-nowrap'
          )}
          href="/app/metrics"
        >
          Metrics
        </Link>
      </li>
      <li
        className={classNames(
          'flex-column flex h-full items-center lg:flex-row',
          pathname === '/app/scaling-components'
            ? 'border-b-4 border-purple-500'
            : 'border-b-4 border-white'
        )}
      >
        <Link
          className={classNames(
            getMenuClassNames(pathname, '/app/scaling-components'),
            'whitespace-nowrap'
          )}
          href="/app/scaling-components"
        >
          Scaling Components
        </Link>
      </li>
      <li className="flex-column flex h-full items-center lg:flex-row">
        <Link
          className={classNames(
            getMenuClassNames(pathname),
            'whitespace-nowrap'
          )}
          href="https://github.com/STCLab-Inc/wave-autoscale"
          target="_blank"
        >
          Github
        </Link>
      </li>
    </ul>
  );
}
