'use client';

import classNames from 'classnames';
import Link from 'next/link';
import { useParams, usePathname } from 'next/navigation';

function getTabClassNames(currentPathname: string, targetPath?: string) {
  return classNames(
    !targetPath || currentPathname.indexOf(targetPath) < 0
      ? 'text-gray-600 pr-4 pl-4'
      : 'text-gray-1000 pr-4 pl-4'
  );
}

function TabItem({
  pathname,
  targetPath,
  label,
}: {
  pathname: string;
  targetPath: string;
  label: string;
}) {
  return (
    <li
      className={classNames(
        'flex-column flex h-full items-center lg:flex-row',
        pathname.includes(targetPath)
          ? 'border-b-4 border-blue-400'
          : 'border-b-4 border-white'
      )}
    >
      <Link
        className={classNames(
          getTabClassNames(pathname, targetPath),
          'whitespace-nowrap'
        )}
        href={targetPath}
      >
        {label}
      </Link>
    </li>
  );
}

export default function PlanningDetailTabs() {
  const pathname = usePathname();
  const { id } = useParams();

  return (
    <div className="flex h-12 w-full items-center border-b px-4">
      <ul className="flex h-full flex-row items-center pt-1 lg:flex-row">
        <TabItem
          pathname={pathname}
          targetPath={`/app/planning/${id}/diagram`}
          label="Diagram"
        />
        <TabItem
          pathname={pathname}
          targetPath={`/app/planning/${id}/code`}
          label="Code"
        />
      </ul>
    </div>
  );
}
