'use client';

import React, { useEffect, useState } from 'react';
import Image from 'next/image';
import { useRouter, usePathname } from 'next/navigation';

import Menu from './menu';

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const router = useRouter();

  const handleClick = () => {
    router.push('/app');
  };

  const pathname = usePathname();

  const [overflowHiddenFlag, setOverflowHiddenFlag] = useState(false);

  useEffect(() => {
    const currentURL = pathname;

    if (
      currentURL.includes('/app/metrics/') ||
      currentURL.includes('/app/metrics/new') ||
      currentURL.includes('/app/scaling-components/') ||
      currentURL.includes('/app/scaling-components/new')
    ) {
      setOverflowHiddenFlag(true);
    } else {
      setOverflowHiddenFlag(false);
    }
  }, [pathname]);

  return (
    <div
      className={`h-screen w-screen min-w-screen-md ${
        overflowHiddenFlag ? 'overflow-hidden' : 'overflow-auto'
      }`}
    >
      <nav className="navbar h-14 w-full p-0 text-neutral-content">
        <div className="flex h-full w-full items-center pl-8 pr-8">
          <div className="flex h-full items-center justify-between">
            <figure onClick={handleClick} className="cursor-pointer">
              <Image
                src="/assets/images/wave-autoscale_symbol.svg"
                alt="wave-autoscale_symbol.svg"
                priority={true}
                width={36}
                height={32}
                style={{
                  minWidth: '2.5rem',
                  maxWidth: '2.5rem',
                }}
              />
            </figure>
            <div className="ml-10 flex h-full items-center">
              <Menu />
            </div>
          </div>
        </div>
      </nav>
      <main className="wa-main">{children}</main>
    </div>
  );
}
