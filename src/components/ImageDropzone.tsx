import React, { useEffect } from 'react'
import { Dropzone, IMAGE_MIME_TYPE } from '@mantine/dropzone'
import {
  IconClipboardCheck,
  IconPhoto,
  IconUpload,
  IconX,
} from '@tabler/icons-react'
import classes from './ImageDropzone.module.css'

interface ImageDropzoneProps {
  onFilesSelected: (files: File[]) => void
  loading?: boolean
}

export const ImageDropzone: React.FC<ImageDropzoneProps> = ({
  onFilesSelected,
  loading = false,
}) => {
  // Support pasting image directly from clipboard (Ctrl+V / Cmd+V)
  useEffect(() => {
    const handlePaste = (e: ClipboardEvent) => {
      const items = e.clipboardData?.items
      if (!items) return

      const pastedFiles: File[] = []
      for (let i = 0; i < items.length; i++) {
        if (items[i].type.startsWith('image/')) {
          const file = items[i].getAsFile()
          if (file) pastedFiles.push(file)
        }
      }

      if (pastedFiles.length > 0) {
        onFilesSelected(pastedFiles)
      }
    }

    window.addEventListener('paste', handlePaste)
    return () => window.removeEventListener('paste', handlePaste)
  }, [onFilesSelected])

  return (
    <div className={classes.container}>
      <Dropzone
        onDrop={onFilesSelected}
        accept={IMAGE_MIME_TYPE}
        maxSize={50 * 1024 * 1024} // 50MB
        loading={loading}
        className={classes.dropzoneRoot}
      >
        <div className={classes.dropzoneContent}>
          <div className={classes.portalGlow}>
            <Dropzone.Accept>
              <div
                className={classes.iconWrapper}
                style={{
                  color: '#10b981',
                  borderColor: 'rgba(16, 185, 129, 0.5)',
                }}
              >
                <IconUpload size={38} stroke={2.2} />
              </div>
            </Dropzone.Accept>
            <Dropzone.Reject>
              <div
                className={classes.iconWrapper}
                style={{
                  color: '#ef4444',
                  borderColor: 'rgba(239, 68, 68, 0.5)',
                }}
              >
                <IconX size={38} stroke={2.2} />
              </div>
            </Dropzone.Reject>
            <Dropzone.Idle>
              <div className={classes.iconWrapper}>
                <IconPhoto size={38} stroke={1.9} />
              </div>
            </Dropzone.Idle>
          </div>

          <div className={classes.textContent}>
            <h2 className={classes.dropzoneTitle}>
              Drop image here, or{' '}
              <span className={classes.browseHighlight}>browse files</span>
            </h2>
            <p className={classes.dropzoneSubtitle}>
              Supports <strong>PNG</strong>, <strong>JPEG</strong>, and{' '}
              <strong>WebP</strong> images with full metadata extraction
            </p>
          </div>

          <div className={classes.tipPills}>
            <span className={classes.tipBadge}>
              <IconClipboardCheck
                size={14}
                style={{ marginRight: 5, verticalAlign: 'middle' }}
              />
              Ctrl+V to paste
            </span>
            <span className={classes.tipBadge}>Batch inspection supported</span>
          </div>
        </div>
      </Dropzone>
    </div>
  )
}
