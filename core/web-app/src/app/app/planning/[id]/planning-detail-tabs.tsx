'use client';

import React, { useEffect } from 'react';
import classNames from 'classnames';
import Link from 'next/link';
import { useParams, usePathname, useRouter } from 'next/navigation';

function getTabClassNames(isActive: boolean) {
  return classNames(
    'flex-column mx-0.5 flex h-full items-center lg:flex-row',
    isActive
      ? 'border-b-4 border-blue-400 text-gray-1000 pr-4 pl-4'
      : 'border-b-4 border-white text-gray-600 pr-4 pl-4'
  );
}

function isActivePath(currentPathname: string, targetPath: string) {
  return currentPathname.includes(targetPath);
}

function TabItem({
  isActive,
  targetPath,
  label,
}: {
  isActive: boolean;
  targetPath: string;
  label: string;
}) {
  return (
    <li className={getTabClassNames(isActive)}>
      <Link className="whitespace-nowrap" href={targetPath}>
        {label}
      </Link>
    </li>
  );
}

export default function PlanningDetailTabs() {
  const pathname = usePathname();
  const { id } = useParams();
  const router = useRouter();

  useEffect(() => {
    const currentURL = pathname;

    if (
      !currentURL.includes('/app/planning/new') &&
      !currentURL.includes('code')
    ) {
      router.push(`/app/planning/${id}/diagram`);
    }
  }, [pathname]);

  const tabItems = [
    { targetPath: `/app/planning/${id}/diagram`, label: 'Diagram' },
    { targetPath: `/app/planning/${id}/code`, label: 'Code' },
  ];

  return (
    <div className="flex h-12 w-full items-center border-b px-4">
      <ul className="flex h-full flex-row items-center pt-1 lg:flex-row">
        {tabItems.map((tab, index) => (
          <TabItem
            key={index}
            isActive={isActivePath(pathname, tab.targetPath)}
            targetPath={tab.targetPath}
            label={tab.label}
          />
        ))}
      </ul>
    </div>
  );
}
