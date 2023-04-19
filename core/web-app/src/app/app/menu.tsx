'use client';

import classNames from 'classnames';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

export default function Menu() {
  const pathname = usePathname();
  console.log('pathname', pathname);
  return (
    <ul
      className="list-style-none mr-auto flex flex-col pl-8 lg:flex-row"
      data-te-navbar-nav-ref
    >
      <li className="my-4 lg:my-0 lg:pr-2" data-te-nav-item-ref>
        <Link
          className={classNames(
            'text-white disabled:text-black/30 lg:px-2 [&.active]:text-black/90 dark:[&.active]:text-neutral-400',
            {
              'font-bold': pathname.indexOf('/app/planning') === 0,
            }
          )}
          href="/app/planning"
          data-te-nav-link-ref
        >
          Planning
        </Link>
      </li>
      <li className="mb-4 lg:mb-0 lg:pr-2" data-te-nav-item-ref>
        <Link
          className={classNames(
            'text-white disabled:text-black/30 lg:px-2 [&.active]:text-black/90 dark:[&.active]:text-neutral-400',
            {
              'font-bold': pathname.indexOf('/app/metrics') === 0,
            }
          )}
          href="/app/metrics"
          data-te-nav-link-ref
        >
          Metrics
        </Link>
      </li>
      <li className="mb-4 lg:mb-0 lg:pr-2" data-te-nav-item-ref>
        <Link
          className={classNames(
            'text-white disabled:text-black/30 lg:px-2 [&.active]:text-black/90 dark:[&.active]:text-neutral-400',
            {
              'font-bold': pathname.indexOf('/app/scaling-components') === 0,
            }
          )}
          href="/app/scaling-components"
          data-te-nav-link-ref
        >
          Scaling Components
        </Link>
      </li>
    </ul>
  );
}
