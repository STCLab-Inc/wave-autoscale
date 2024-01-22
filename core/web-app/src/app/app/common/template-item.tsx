import { Template } from '@/data/wa-templates';
import DefinitionService from '@/services/definition';
import { errorToast } from '@/utils/toast';
import dynamic from 'next/dynamic';
import { useRouter } from 'next/navigation';

const YAMLEditor = dynamic(() => import('./yaml-editor'), {
  ssr: false,
});

function TemplateContent({ template }: { template: Template }) {
  return (
    <div className="flex flex-col">
      {/* Icons */}
      <div className="icon mb-2 flex flex-row space-x-4">
        <div className="flex h-16 w-16 items-center justify-center">
          <img
            src={template.image1}
            alt={template.title}
            className="min-h-12 max-h-12 max-w-[3rem] object-contain"
          />
        </div>
        {template.image2 && (
          <div className="flex h-16 w-16 items-center justify-center">
            <img
              src={template.image2}
              alt={template.title}
              className="min-h-12 max-h-12 max-w-[3rem] object-contain"
            />
          </div>
        )}
      </div>
      <h2 className="card-title mb-2 text-sm font-medium text-wa-gray-700">
        {template.title}
      </h2>
      <p className="text-wrap mb-6 line-clamp-3 h-16 text-sm font-normal text-wa-gray-700">
        {template.description}
      </p>
      <div className="card-actions justify-start">
        <label
          className="btn-gray btn flex !h-10 !min-h-[2.5rem] !w-[138px] items-center justify-center !rounded-lg !text-sm !font-medium !normal-case"
          htmlFor={template.title}
        >
          <img
            src="/assets/template-item/code.svg"
            alt="code"
            className="mr-1 h-5 w-5"
          />
          View Code
        </label>
      </div>
    </div>
  );
}

export function TemplateItem({
  template,
  isCard = false,
}: {
  template: Template;
  isCard: boolean;
}) {
  const { push } = useRouter();

  const handleAddPlan = async (code: string) => {
    console.log(code);

    try {
      const response = await DefinitionService.createDefinitions(code);
    } catch (error: any) {
      console.log(error);
      errorToast(error.message);
      return;
    }
    if (template.metricsOnly) {
      push('/app/metrics');
      return;
    }
    push('/app/scaling-plans');
  };
  return (
    <>
      {/* CardView or not */}
      {isCard ? (
        <div className="wa-card">
          <div className="wa-card-body">
            <TemplateContent template={template} />
          </div>
        </div>
      ) : (
        <TemplateContent template={template} />
      )}
      <input type="checkbox" id={template.title} className="modal-toggle" />
      <div className="modal">
        <div className="modal-box flex h-[700px] w-11/12 max-w-5xl flex-col">
          <h3 className="text-lg font-bold">{template.title}</h3>
          <p className="py-4 text-base font-normal">{template.description}</p>
          <div className="min-h-0 flex-1">
            <YAMLEditor value={template.code} showLineNumbers readonly />
          </div>
          <div className="modal-action">
            <label
              htmlFor={template.title}
              className="btn-primary btn-sm btn"
              onClick={() => handleAddPlan(template.code)}
            >
              Add Definitions
            </label>
            <label htmlFor={template.title} className="btn-sm btn">
              Close
            </label>
          </div>
        </div>
      </div>
    </>
  );
}
