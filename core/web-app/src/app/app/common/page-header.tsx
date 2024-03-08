import { useRouter } from 'next/navigation';

interface PageHeaderProps {
  title: string;
  subtitle?: string;
  backButton?: boolean;
  backUrl?: string;
}
export default function PageHeader(props: PageHeaderProps) {
  const router = useRouter();

  return (
    <div className="flex h-14 min-h-14 w-full items-center border-b border-wa-gray-200 bg-white px-6">
      {props.backButton && (
        <div className="flex items-center">
          <button
            className="btn-ghost btn mr-2 w-10 p-2"
            onClick={() => {
              if (props.backUrl) {
                router.push(props.backUrl);
                return;
              }
              router.back();
            }}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth="1.5"
              stroke="currentColor"
              className="h-6 w-6"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18"
              />
            </svg>
          </button>
        </div>
      )}

      <div className="text-[1rem] font-semibold">{props.title}</div>
      {props.subtitle && (
        <>
          {/* divider */}
          <div className="mx-[10px] h-6 w-[1px] bg-wa-gray-300" />
          <div className="text-sm text-wa-gray-600">{props.subtitle}</div>
        </>
      )}
    </div>
  );
}
