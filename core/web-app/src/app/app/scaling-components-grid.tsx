import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';
import classNames from 'classnames';
import { memo, useMemo } from 'react';

interface ScalingComponentMetadata {
  id: string;
  title: string;
  icon?: string;
}

const SCALING_COMPONENTS_METADATA: ScalingComponentMetadata[] = [
  {
    id: 'kubernetes-deployment',
    title: 'Kubernetes Deployment',
    icon: '/assets/app-icons/kubernetes.svg',
  },
  {
    id: 'kubernetes-json-patch',
    title: 'Kubernetes JSON Patch',
    icon: '/assets/app-icons/kubernetes.svg',
  },
  {
    id: 'amazon-ecs',
    title: 'Amazon ECS',
    icon: '/assets/app-icons/aws-ecs.svg',
  },
  {
    id: 'aws-ec2-autoscaling',
    title: 'AWS EC2 Autoscaling',
    icon: '/assets/app-icons/aws-ec2-autoscaling.svg',
  },
  {
    id: 'aws-lambda',
    title: 'AWS Lambda',
    icon: '/assets/app-icons/aws-lambda.svg',
  },
  {
    id: 'aws-wafv2',
    title: 'AWS WAFv2',
    icon: '/assets/app-icons/aws-waf.svg',
  },
  {
    id: 'amazon-emr-ec2',
    title: 'Amazon EMR EC2',
    icon: '/assets/app-icons/aws-emr.svg',
  },
  {
    id: 'amazon-dynamodb',
    title: 'Amazon DynamoDB',
    icon: '/assets/app-icons/aws-dynamodb.svg',
  },
  {
    id: 'google-cloud-run',
    title: 'Google Cloud Run',
    icon: '/assets/app-icons/gcp-cloud-run.svg',
  },
  {
    id: 'google-cloud-functions',
    title: 'Google Cloud Functions',
    icon: '/assets/app-icons/gcp-cloud-functions.svg',
  },
  {
    id: 'gcp-compute-engine-mig',
    title: 'GCP Compute Engine MIG',
    icon: '/assets/app-icons/gcp-compute-engine.svg',
  },
  {
    id: 'azure-virtual-machine-scale-sets',
    title: 'Azure VMSS',
    icon: '/assets/app-icons/azure-vmss.svg',
  },
  {
    id: 'azure-functions',
    title: 'Azure Functions',
    icon: '/assets/app-icons/azure-functions.svg',
  },
  {
    id: 'cloudflare-rule',
    title: 'Cloudflare Rule',
    icon: '/assets/app-icons/cloudflare.svg',
  },
  {
    id: 'netfunnel',
    title: 'NetFunnel',
    icon: '/assets/app-icons/netfunnel.png',
  },
  {
    id: 'wa-logger',
    title: 'Wave Autoscale Logger',
    // icon: '/assets/app-icons/wa-logger.svg',
  },
];

function ScalingComponentGridItem({
  metadata,
  scalingComponents,
  isLoading,
}: {
  metadata: any;
  scalingComponents: ScalingComponentDefinition[];
  isLoading: boolean;
}) {
  return (
    <div className="stats">
      <div className="stat">
        {/* Title */}
        <div className="mb-4 flex items-center justify-start">
          <div
            className={classNames(
              'text-wrap stat-title flex-1 overflow-hidden',
              {
                skeleton: isLoading,
              }
            )}
          >
            {metadata.title}
          </div>
          <div
            className={classNames(
              'stat-icon ml-2 flex h-8 w-8 items-center justify-center',
              {
                skeleton: isLoading,
              }
            )}
          >
            {metadata.icon && (
              <img
                className="max-h-[32px] max-w-[32px]"
                src={metadata.icon}
                alt={metadata.title}
              />
            )}
          </div>
        </div>
        <div
          className={classNames('stat-value flex h-[100px] overflow-hidden', {
            skeleton: isLoading,
          })}
        >
          {!isLoading && !scalingComponents?.length ? (
            // Connect Button
            <div className="flex flex-1 items-center justify-center">
              <a href={`/app/scaling-components`} className="btn btn-sm">
                Connect
              </a>
            </div>
          ) : (
            // Scaling Components
            <div className="flex flex-col items-start justify-start overflow-y-auto">
              {scalingComponents.map((scalingComponent) => (
                <div
                  key={scalingComponent.id}
                  className="flex flex-col items-center justify-center"
                >
                  <div className="break-all text-center text-xs">
                    {scalingComponent.id}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function ScalingComponentsGrid({
  scalingComponents,
  isLoading,
}: {
  scalingComponents: ScalingComponentDefinition[];
  isLoading: boolean;
}) {
  // Show first the connected scaling components
  const orderedMetadata = useMemo(() => {
    const ordered: ScalingComponentMetadata[] = [];
    SCALING_COMPONENTS_METADATA.forEach((metadata) => {
      if (
        scalingComponents.find(
          (scalingComponent) => scalingComponent.component_kind === metadata.id
        )
      ) {
        ordered.splice(0, 0, metadata);
        return;
      }
      ordered.push(metadata);
    });
    return ordered;
  }, [scalingComponents]);
  return (
    <div className="grid gap-4 md:grid-cols-3 lg:grid-cols-4 2xl:grid-cols-7">
      {!isLoading &&
        orderedMetadata.map((metadata) => {
          const filtered = scalingComponents.filter(
            (scalingComponent) =>
              scalingComponent.component_kind === metadata.id
          );
          return (
            <ScalingComponentGridItem
              key={metadata.id}
              metadata={metadata}
              scalingComponents={filtered}
              isLoading={false}
            />
          );
        })}
      {/* Loading */}
      {isLoading &&
        Array.from({ length: SCALING_COMPONENTS_METADATA.length }).map(
          (_, index) => (
            <ScalingComponentGridItem
              key={index}
              metadata={{}}
              scalingComponents={[]}
              isLoading={true}
            />
          )
        )}
    </div>
  );
}

export default memo(ScalingComponentsGrid);
