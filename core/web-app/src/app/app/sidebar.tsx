import Link from 'next/link';
import Menu from './common/menu';
import Image from 'next/image';
import { usePathname, useRouter } from 'next/navigation';
import { useInfo } from '@/services/info';

export default function Sidebar() {
  const router = useRouter();
  const pathname = usePathname();
  const { data: info } = useInfo();

  return (
    <nav className="sidebar min-w-64 fixed left-0 top-0 flex h-screen w-64 flex-col border-r border-wa-gray-500 bg-wa-gray-50 z-50">
      {/* Logo */}
      <figure
        onClick={() => router.push('/app')}
        className="h-14 w-full border-b border-wa-gray-200 bg-white px-4 py-3"
      >
        <Image
          src="/assets/sidebar/logo.svg"
          alt="Logo"
          priority={true}
          width={165}
          height={32}
          // style={{
          //   minWidth: '2.5rem',
          //   maxWidth: '2.5rem',
          // }}
        />
      </figure>
      {/* Menu */}
      <Menu />
      {/* Info */}
      <div className="my-4">
        <div className="flex w-full flex-row items-center justify-center space-x-2">
          <div className="text-xs text-wa-gray-700">Version</div>
          <div className="text-xs text-wa-gray-700">{info?.version}</div>
        </div>
      </div>
      {/* Github */}
      <div className="mb-4 flex h-14 w-full border-t border-wa-gray-200">
        <Link
          href="https://github.com/stclab-inc/wave-autoscale"
          target="_blank"
          className="flex w-full items-center justify-start px-6"
        >
          <img src="/assets/sidebar/github.svg" alt="Github" />
          <span className="mx-2 flex-1 text-sm text-wa-gray-700">Github</span>
          <img src="/assets/sidebar/external.svg" alt="Open" />
        </Link>
      </div>
    </nav>
  );
}
