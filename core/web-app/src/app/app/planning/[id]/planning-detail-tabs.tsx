'use client';

import classNames from 'classnames';
import Link from 'next/link';
import { useParams, usePathname } from 'next/navigation';

function getActiveTabClassNames(pathname: string, tabPath: string) {
  return classNames('tab no-underline mr-3', {
    'tab-active font-bold link-primary tab-bordered':
      pathname.indexOf(tabPath) > 0,
  });
}

export default function PlanningDetailTabs() {
  const pathname = usePathname();
  const { id } = useParams();
  return (
    <div className="tabs ml-3">
      <Link
        className={getActiveTabClassNames(pathname, '/diagram')}
        href={`/app/planning/${id}/diagram`}
      >
        Diagram
      </Link>
      <Link
        className={getActiveTabClassNames(pathname, '/code')}
        href={`/app/planning/${id}/code`}
      >
        Code
      </Link>
    </div>
  );
}
