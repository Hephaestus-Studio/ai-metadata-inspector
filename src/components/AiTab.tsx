import React from 'react'
import {
  Accordion,
  ActionIcon,
  Badge,
  Button,
  CopyButton,
  Group,
  Stack,
  Text,
  Tooltip,
} from '@mantine/core'
import { notifications } from '@mantine/notifications'
import {
  IconCheck,
  IconCode,
  IconCopy,
  IconCpu,
  IconDownload,
  IconSparkles,
} from '@tabler/icons-react'
import clsx from 'clsx'
import type { AiMetadata, ImageMetadataReport } from '@/types/metadata'
import { TabEmptyState } from './TabEmptyState'
import classes from './AiTab.module.css'

interface AiTabProps {
  ai?: AiMetadata
  report?: ImageMetadataReport
  onNavigateTab?: (tab: string) => void
}

export const AiTab: React.FC<AiTabProps> = ({ ai, report, onNavigateTab }) => {
  if (!ai || (!ai.prompt && !ai.platform && !ai.comfy_workflow_json)) {
    return (
      <TabEmptyState
        icon={<IconCpu size={28} stroke={1.5} />}
        title="No AI Generation Prompts Detected"
        description="This image does not contain embedded Stable Diffusion parameters, ComfyUI node workflows, NovelAI headers, or Midjourney prompts."
        currentTab="ai"
        report={report}
        onNavigateTab={onNavigateTab}
      />
    )
  }

  const handleDownloadWorkflow = () => {
    const jsonStr = ai.comfy_workflow_json || ai.comfy_prompt_json
    if (!jsonStr) return

    const blob = new Blob([jsonStr], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'comfyui_workflow.json'
    a.click()
    URL.revokeObjectURL(url)

    notifications.show({
      title: 'Workflow Exported',
      message: 'ComfyUI Workflow JSON downloaded successfully!',
      color: 'teal',
      icon: <IconCheck size={16} />,
    })
  }

  return (
    <Stack gap="md">
      {/* Platform Header */}
      <Group justify="space-between" align="center">
        <Group gap="xs">
          <Badge
            variant="gradient"
            gradient={{ from: 'indigo', to: 'cyan' }}
            size="lg"
            leftSection={<IconSparkles size={14} />}
          >
            {ai.platform || 'Generative AI'}
          </Badge>
          {ai.size && (
            <Badge variant="outline" color="gray" size="md">
              {ai.size}
            </Badge>
          )}
        </Group>

        {(ai.comfy_workflow_json || ai.comfy_prompt_json) && (
          <Button
            size="xs"
            variant="light"
            color="cyan"
            leftSection={<IconDownload size={14} />}
            onClick={handleDownloadWorkflow}
          >
            Download ComfyUI Workflow (.json)
          </Button>
        )}
      </Group>

      {/* Positive Prompt */}
      {ai.prompt && (
        <div>
          <Group justify="space-between" align="center">
            <Text size="xs" fw={700} c="dimmed" tt="uppercase">
              Positive Prompt
            </Text>
            <CopyButton value={ai.prompt}>
              {({ copied, copy }) => (
                <Tooltip label={copied ? 'Copied!' : 'Copy Prompt'}>
                  <ActionIcon
                    variant="subtle"
                    color={copied ? 'teal' : 'gray'}
                    size="sm"
                    onClick={copy}
                  >
                    {copied ? <IconCheck size={14} /> : <IconCopy size={14} />}
                  </ActionIcon>
                </Tooltip>
              )}
            </CopyButton>
          </Group>
          <div className={classes.promptBox}>{ai.prompt}</div>
        </div>
      )}

      {/* Negative Prompt */}
      {ai.negative_prompt && (
        <div>
          <Group justify="space-between" align="center">
            <Text size="xs" fw={700} c="red" tt="uppercase">
              Negative Prompt
            </Text>
            <CopyButton value={ai.negative_prompt}>
              {({ copied, copy }) => (
                <Tooltip label={copied ? 'Copied!' : 'Copy Negative Prompt'}>
                  <ActionIcon
                    variant="subtle"
                    color={copied ? 'teal' : 'gray'}
                    size="sm"
                    onClick={copy}
                  >
                    {copied ? <IconCheck size={14} /> : <IconCopy size={14} />}
                  </ActionIcon>
                </Tooltip>
              )}
            </CopyButton>
          </Group>
          <div className={clsx(classes.promptBox, classes.negativePromptBox)}>
            {ai.negative_prompt}
          </div>
        </div>
      )}

      {/* Hyperparameters Grid */}
      <div className={classes.paramsGrid}>
        {ai.steps !== undefined && (
          <div className={classes.paramCard}>
            <div className={classes.paramLabel}>Sampling Steps</div>
            <div className={classes.paramValue}>{ai.steps}</div>
          </div>
        )}

        {ai.sampler && (
          <div className={classes.paramCard}>
            <div className={classes.paramLabel}>Sampler</div>
            <div className={classes.paramValue}>{ai.sampler}</div>
          </div>
        )}

        {ai.cfg_scale !== undefined && (
          <div className={classes.paramCard}>
            <div className={classes.paramLabel}>CFG Scale</div>
            <div className={classes.paramValue}>{ai.cfg_scale}</div>
          </div>
        )}

        {ai.seed !== undefined && (
          <div className={classes.paramCard}>
            <div className={classes.paramLabel}>Seed</div>
            <div className={classes.paramValue}>{ai.seed}</div>
          </div>
        )}

        {ai.model_name && (
          <div className={classes.paramCard} style={{ gridColumn: 'span 2' }}>
            <div className={classes.paramLabel}>Model / Checkpoint</div>
            <div className={classes.paramValue}>{ai.model_name}</div>
          </div>
        )}

        {ai.model_hash && (
          <div className={classes.paramCard}>
            <div className={classes.paramLabel}>Model Hash</div>
            <div className={classes.paramValue}>{ai.model_hash}</div>
          </div>
        )}

        {ai.denoising_strength !== undefined && (
          <div className={classes.paramCard}>
            <div className={classes.paramLabel}>Denoising Strength</div>
            <div className={classes.paramValue}>{ai.denoising_strength}</div>
          </div>
        )}

        {ai.clip_skip !== undefined && (
          <div className={classes.paramCard}>
            <div className={classes.paramLabel}>Clip Skip</div>
            <div className={classes.paramValue}>{ai.clip_skip}</div>
          </div>
        )}
      </div>

      {/* LoRA Tags */}
      {ai.lora_tags && ai.lora_tags.length > 0 && (
        <div>
          <Text size="xs" fw={700} c="dimmed" tt="uppercase" mb={6}>
            LoRA Models ({ai.lora_tags.length})
          </Text>
          <Group gap="xs">
            {ai.lora_tags.map((tag, idx) => (
              <Badge key={idx} variant="dot" color="indigo" size="sm">
                {tag}
              </Badge>
            ))}
          </Group>
        </div>
      )}

      {/* ComfyUI Raw Graph Viewer Accordion */}
      {(ai.comfy_prompt_json || ai.comfy_workflow_json) && (
        <Accordion variant="separated" radius="md" mt="xs">
          <Accordion.Item value="comfy-json">
            <Accordion.Control icon={<IconCode size={18} color="#0ea5e9" />}>
              <Text size="sm" fw={600}>
                View ComfyUI Node Execution Graph (JSON)
              </Text>
            </Accordion.Control>
            <Accordion.Panel>
              <pre className={classes.jsonViewer}>
                <code>{ai.comfy_prompt_json || ai.comfy_workflow_json}</code>
              </pre>
            </Accordion.Panel>
          </Accordion.Item>
        </Accordion>
      )}
    </Stack>
  )
}
