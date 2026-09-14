import React from 'react'
import {
  ActionIcon,
  Badge,
  Box,
  Button,
  Group,
  Modal,
  Text,
  useMantineColorScheme,
} from '@mantine/core'
import { useDisclosure } from '@mantine/hooks'
import {
  IconBrandGithub,
  IconDownload,
  IconEyeCode,
  IconHelpCircle,
  IconMoon,
  IconPlus,
  IconSun,
  IconTrash,
  IconX,
} from '@tabler/icons-react'
import clsx from 'clsx'
import type { BatchItem } from './BatchQueue'
import classes from './Header.module.css'

interface HeaderProps {
  items?: BatchItem[]
  selectedIndex?: number
  onSelectIndex?: (index: number) => void
  onRemoveItem?: (index: number) => void
  onClearAll?: () => void
  onAddMore?: () => void
  onBatchCleanAndDownloadZip?: () => void
  cleaningBatch?: boolean
}

export const Header: React.FC<HeaderProps> = ({
  items = [],
  selectedIndex = 0,
  onSelectIndex,
  onRemoveItem,
  onClearAll,
  onAddMore,
  onBatchCleanAndDownloadZip,
  cleaningBatch = false,
}) => {
  const { colorScheme, toggleColorScheme } = useMantineColorScheme()
  const isDark = colorScheme === 'dark'
  const [aboutOpened, { open: openAbout, close: closeAbout }] =
    useDisclosure(false)

  const hasMultipleItems = items.length > 1
  const totalSizeMb = (
    items.reduce((acc, it) => acc + it.file.size, 0) /
    (1024 * 1024)
  ).toFixed(1)

  return (
    <header className={classes.headerContainer}>
      <div className={classes.inner}>
        {/* Left Floating Brand Island */}
        <div className={classes.brandIsland}>
          <div className={classes.logoIcon}>
            <IconEyeCode size={20} stroke={2.2} />
          </div>
          <div>
            <h1 className={classes.title}>AI Metadata Inspector</h1>
            <div className={classes.subtitle}>
              Deep AI Prompts & Privacy Stripper
            </div>
          </div>
        </div>

        {/* Center Batch Queue Filmstrip (When multiple images exist) */}
        {hasMultipleItems && onSelectIndex && (
          <div className={classes.centerBatchDock}>
            <div className={classes.chipsScrollArea}>
              {items.map((item, idx) => {
                const isActive = idx === selectedIndex
                return (
                  <div
                    key={item.id}
                    className={clsx(
                      classes.itemChip,
                      isActive && classes.itemChipActive,
                    )}
                    onClick={() => onSelectIndex(idx)}
                  >
                    <div className={classes.chipThumbnailWrapper}>
                      <img
                        src={item.previewUrl}
                        alt={item.file.name}
                        className={classes.chipThumbnail}
                      />
                    </div>
                    <span className={classes.chipFileName}>
                      {item.file.name}
                    </span>
                    {item.report && (
                      <Badge
                        size="xs"
                        variant="filled"
                        color={
                          item.report.risk_report.level === 'safe'
                            ? 'teal'
                            : item.report.risk_report.level === 'medium'
                              ? 'orange'
                              : 'red'
                        }
                      >
                        {item.report.risk_report.score}
                      </Badge>
                    )}
                    {onRemoveItem && (
                      <ActionIcon
                        size="xs"
                        variant="subtle"
                        color="gray"
                        onClick={(e) => {
                          e.stopPropagation()
                          onRemoveItem(idx)
                        }}
                      >
                        <IconX size={12} />
                      </ActionIcon>
                    )}
                  </div>
                )
              })}

              {onAddMore && (
                <Button
                  size="xs"
                  variant="subtle"
                  color="indigo"
                  leftSection={<IconPlus size={14} />}
                  onClick={onAddMore}
                  className={classes.addChipBtn}
                >
                  Add
                </Button>
              )}
            </div>
          </div>
        )}

        {/* Right Floating Actions Island */}
        <div className={classes.actionsIsland}>
          <Group gap={6} wrap="nowrap">
            {/* Batch Clean ZIP Button */}
            {hasMultipleItems && onBatchCleanAndDownloadZip && (
              <Button
                size="xs"
                variant="filled"
                color="teal"
                loading={cleaningBatch}
                leftSection={<IconDownload size={14} />}
                onClick={onBatchCleanAndDownloadZip}
                className={classes.batchCleanBtn}
              >
                Clean All ({items.length} files • {totalSizeMb} MB)
              </Button>
            )}

            {hasMultipleItems && onClearAll && (
              <ActionIcon
                className={classes.glassButton}
                size="md"
                color="red"
                onClick={onClearAll}
                title="Clear all images"
                aria-label="Clear all images"
              >
                <IconTrash size={16} color="#ef4444" />
              </ActionIcon>
            )}

            {items.length === 1 && onAddMore && (
              <Button
                size="xs"
                variant="light"
                color="indigo"
                leftSection={<IconPlus size={14} />}
                onClick={onAddMore}
                className={classes.addFileBtn}
              >
                Add Image
              </Button>
            )}

            <ActionIcon
              className={classes.glassButton}
              size="md"
              onClick={openAbout}
              aria-label="About"
            >
              <IconHelpCircle size={18} />
            </ActionIcon>

            <ActionIcon
              component="a"
              href="https://github.com/hephaestus-studio/ai-metadata-inspector"
              target="_blank"
              rel="noopener noreferrer"
              className={classes.glassButton}
              size="md"
              aria-label="GitHub Repository"
            >
              <IconBrandGithub size={18} />
            </ActionIcon>

            <ActionIcon
              className={classes.glassButton}
              size="md"
              onClick={() => toggleColorScheme()}
              aria-label="Toggle Color Scheme"
            >
              {isDark ? (
                <IconSun size={18} color="#38bdf8" />
              ) : (
                <IconMoon size={18} color="#8b5cf6" />
              )}
            </ActionIcon>
          </Group>
        </div>
      </div>

      <Modal
        opened={aboutOpened}
        onClose={closeAbout}
        title="About AI Metadata Inspector"
        centered
        radius="lg"
      >
        <Box p="xs">
          <Text size="sm" mb="md">
            <strong>AI Metadata Inspector & Privacy Stripper</strong> is an
            open-source, ultra-fast client-side analyzer built with{' '}
            <strong>Rust WebAssembly</strong> and React.
          </Text>
          <Text size="sm" mb="sm" fw={600}>
            🛡️ Privacy First:
          </Text>
          <Text size="xs" c="dimmed" mb="md">
            All inspection, extraction, and lossless cleaning happen directly in
            your web browser. Your images are <strong>never uploaded</strong> to
            any server or third party.
          </Text>
          <Text size="sm" mb="sm" fw={600}>
            ✨ Supported Ecosystems:
          </Text>
          <Text size="xs" c="dimmed" mb="lg">
            • Stable Diffusion WebUI / Automatic1111 / Forge / SD.Next
            <br />
            • ComfyUI Node Graphs & Workflows (FLUX, SDXL, SD 1.5)
            <br />
            • NovelAI, SwarmUI, InvokeAI, Fooocus, Midjourney, DALL-E
            <br />
            • EXIF Camera Data, GPS Geolocation Coordinates
            <br />• C2PA / Content Authenticity Initiative (CAI) Credentials
          </Text>
          <Group justify="flex-end">
            <Button variant="light" onClick={closeAbout}>
              Got it!
            </Button>
          </Group>
        </Box>
      </Modal>
    </header>
  )
}
