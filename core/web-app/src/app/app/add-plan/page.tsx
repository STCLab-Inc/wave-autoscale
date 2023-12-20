'use client';

import React from 'react';
import ContentHeader from '../common/content-header';
import TEMPLATES, {
  Template,
  TemplateSection,
} from '../../../data/wa-templates';
import YAMLEditor from '../common/yaml-editor';
import DefinitionService from '@/services/definition';
import { useRouter } from 'next/navigation';

const ALL_TEMPLATES = [...TEMPLATES.SCALING_COMPONENTS, ...TEMPLATES.METRICS];

function TemplateSection({
  templateSection,
}: {
  templateSection: TemplateSection;
}) {
  return (
    <div className="flex flex-col px-10 py-8">
      <h3 className="mb-4 text-lg font-semibold" id={templateSection.title}>
        {templateSection.title}
      </h3>
      <div className="grid grid-cols-3 gap-4 lg:grid-cols-6">
        {templateSection.templates.map((template) => (
          <TemplateCard key={template.title} template={template} />
        ))}
      </div>
    </div>
  );
}

function TemplateCard({ template }: { template: Template }) {
  const router = useRouter();
  const [code, setCode] = React.useState(template.code);

  // Add the code including Metric Plans, Scaling Plans, and Scaling Components to the /api/definitions
  const handleAddPlan = async (code: string) => {
    console.log(code);

    try {
      const response = await DefinitionService.createDefinitions(code);
    } catch (error: any) {
      console.log(error);
      alert(error.message);
      return;
    }
    if (template.metricsOnly) {
      router.push('/app/metrics');
      return;
    }
    router.push('/app/scaling-plans');
  };

  return (
    <>
      <div className="card bg-base-100 shadow-xl">
        <div className="card-body">
          <div className="icon flex flex-row space-x-4">
            <img
              src={template.image1}
              alt={template.title}
              className="min-h-12 max-h-12 max-w-[3rem] object-contain"
            />
            {template.image2 && (
              <img
                src={template.image2}
                alt={template.title}
                className="min-h-12 max-h-12 max-w-[3rem] object-contain"
              />
            )}
          </div>
          <h2 className="card-title text-base">{template.title}</h2>
          <p className="text-sm">{template.description}</p>
          <div className="card-actions justify-start">
            <label className="btn-primary btn-sm btn" htmlFor={template.title}>
              View Code
            </label>
          </div>
        </div>
      </div>
      <input type="checkbox" id={template.title} className="modal-toggle" />
      <div className="modal">
        <div className="modal-box w-11/12 max-w-5xl">
          <h3 className="text-lg font-bold">{template.title}</h3>
          <p className="py-4">{template.description}</p>
          <YAMLEditor
            value={code}
            onChange={(value) => setCode(value)}
            showLineNumbers
          />
          <div className="modal-action">
            <label
              htmlFor={template.title}
              className="btn-primary btn"
              onClick={() => handleAddPlan(code)}
            >
              Add Plan
            </label>
            <label htmlFor={template.title} className="btn">
              Close
            </label>
          </div>
        </div>
      </div>
    </>
  );
}

export default function AddPlanPage() {
  return (
    <main className="flex h-full w-full flex-col">
      {/* Header */}
      <ContentHeader
        type="OUTER"
        title="Add Plan"
        right={<div>Add a plan with a variety of templates </div>}
      />
      {/* Shortcuts for Templates */}
      <div className="flex flex-row items-center space-x-2 px-10 py-8">
        <span className="font-bold">Scaling Components</span>
        {TEMPLATES.SCALING_COMPONENTS.map((templateSection) => (
          <a
            key={templateSection.title}
            href={`#${templateSection.title}`}
            className="btn-link btn-sm btn"
          >
            {templateSection.title}
          </a>
        ))}
        <span className="!ml-12 font-bold">Metrics</span>
        {TEMPLATES.METRICS.map((templateSection) => (
          <a
            key={templateSection.title}
            href={`#${templateSection.title}`}
            className="btn-link btn-sm btn"
          >
            {templateSection.title}
          </a>
        ))}
      </div>
      {/* Templates Grid */}
      {ALL_TEMPLATES.map((templateSection) => (
        <TemplateSection
          key={templateSection.title}
          templateSection={templateSection}
        />
      ))}
    </main>
  );
}
