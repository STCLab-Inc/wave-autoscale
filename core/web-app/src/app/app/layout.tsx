'use client';

import React, { useEffect, useRef, useState } from 'react';
import Image from 'next/image';
import { useRouter } from 'next/navigation';
import { usePathname } from 'next/navigation';
import Menu from './common/menu';

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();

  // const [windowWidth, setWindowWidth] = useState<number>(0);

  // useEffect(() => {
  //   const handleResize = () => {
  //     setWindowWidth(window.innerWidth);
  //   };

  //   setWindowWidth(window.innerWidth);
  //   window.addEventListener('resize', handleResize);

  //   return () => {
  //     window.removeEventListener('resize', handleResize);
  //   };
  // }, []);

  // const [menuFlag, handleClickMenu] = useState<boolean>(false);

  // useEffect(() => {
  //   if (windowWidth >= 880) {
  //     handleClickMenu(false);
  //   }
  // }, [windowWidth]);

  // useEffect(() => {
  //   handleClickMenu(false);
  // }, [pathname]);

  // const menuRef = useRef<HTMLInputElement>(null);

  // useEffect(() => {
  //   const clickOutside = (event: any) => {
  //     if (
  //       menuFlag &&
  //       menuRef.current &&
  //       !menuRef.current?.contains(event.target)
  //     ) {
  //       handleClickMenu(false);
  //     }
  //   };
  //   document.addEventListener('mousedown', clickOutside);
  //   return () => {
  //     document.removeEventListener('mousedown', clickOutside);
  //   };
  // }, [menuFlag]);

  return (
    <div className="h-screen w-screen min-w-screen-md overflow-auto">
      <nav
        className="navbar relative w-full px-10 py-0 text-neutral-content h-0 min-h-16"
        // ref={menuRef}
      >
        {/* Logo */}
        <figure
          onClick={() => router.push('/app')}
          className="mr-10 cursor-pointer"
        >
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
        {/* Menu */}
        <Menu />
        {/* {menuFlag && (
          <div className="absolute top-16 z-30 flex w-full flex-col items-center overflow-y-auto border-b border-gray-200 bg-base-100 px-8 py-8">
            <Menu
              type="FOLD"
              windowWidth={windowWidth}
              menuFlag={menuFlag}
              onClickMenu={setMenuFlag}
            />
          </div>
        )} */}
      </nav>
      <main className="wa-main">{children}</main>
    </div>
  );
}
