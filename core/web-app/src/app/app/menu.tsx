'use client';

import classNames from 'classnames';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

// TODO
function getMenuClassNames(currentPathname: string, targetPath?: string) {
  const COMMON_CLASS_NAMES =
    'text-white disabled:text-black/30 lg:px-2 [&.active]:text-black/90 dark:[&.active]:text-neutral-400';
  if (!targetPath) {
    return COMMON_CLASS_NAMES;
  }
  return classNames(COMMON_CLASS_NAMES, {
    'tab-active font-bold link-primary tab-bordered':
      currentPathname.indexOf(targetPath) >= 0,
  });
}

export default function Menu() {
  const pathname = usePathname();
  return (
    <ul
      className="list-style-none mr-auto flex flex-col pl-8 lg:flex-row"
      data-te-navbar-nav-ref
    >
      <li className="my-4 lg:my-0 lg:pr-2" data-te-nav-item-ref>
        <Link
          className={getMenuClassNames(pathname, '/app/planning')}
          href="/app/planning"
          data-te-nav-link-ref
        >
          Planning
        </Link>
      </li>
      <li className="mb-4 lg:mb-0 lg:pr-2" data-te-nav-item-ref>
        <Link
          className={getMenuClassNames(pathname, '/app/metrics')}
          href="/app/metrics"
          data-te-nav-link-ref
        >
          Metrics
        </Link>
      </li>
      <li className="mb-4 lg:mb-0 lg:pr-2" data-te-nav-item-ref>
        <Link
          className={getMenuClassNames(pathname, '/app/scaling-components')}
          href="/app/scaling-components"
          data-te-nav-link-ref
        >
          Scaling Components
        </Link>
      </li>
      <li className="mb-4 lg:mb-0 lg:pr-2" data-te-nav-item-ref>
        <Link
          className={getMenuClassNames(pathname)}
          href="https://github.com/STCLab-Inc/wave-autoscale"
          target="_blank"
          data-te-nav-link-ref
        >
          Github
        </Link>
      </li>
    </ul>
  );
}
