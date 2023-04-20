'use client';

import classNames from 'classnames';
import { usePathname } from 'next/navigation';

function getActiveTabClassNames(pathname: string, tabPath: string) {
  return classNames('tab no-underline mr-3', {
    'tab-active font-bold link-primary tab-bordered':
      pathname.indexOf(tabPath) > 0,
  });
}

export default function PlanningDetailTabs() {
  const pathname = usePathname();
  return (
    <div className="tabs ml-3">
      <a className={getActiveTabClassNames(pathname, '/diagram')}>Diagram</a>
      <a className={getActiveTabClassNames(pathname, '/code')}>Code</a>
    </div>
  );
}
