import React, { useEffect, useRef } from 'react';

export const CyberCanvas: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let animationId: number;
    let width = (canvas.width = window.innerWidth);
    let height = (canvas.height = window.innerHeight);

    let mouseX = width * 0.7;
    let mouseY = height * 0.35;
    let targetX = mouseX;
    let targetY = mouseY;

    const handleResize = () => {
      width = canvas.width = window.innerWidth;
      height = canvas.height = window.innerHeight;
    };

    const handleMouseMove = (e: MouseEvent) => {
      targetX = e.clientX;
      targetY = e.clientY;
    };

    window.addEventListener('resize', handleResize);
    window.addEventListener('mousemove', handleMouseMove);

    // Particle nodes
    const particleCount = 60;
    const particles = Array.from({ length: particleCount }, () => ({
      x: Math.random() * width,
      y: Math.random() * height,
      vx: (Math.random() - 0.5) * 0.4,
      vy: (Math.random() - 0.5) * 0.4,
      size: Math.random() * 2 + 1,
      alpha: Math.random() * 0.5 + 0.2,
    }));

    let frame = 0;

    const render = () => {
      frame++;
      // Smooth mouse interpolation
      mouseX += (targetX - mouseX) * 0.05;
      mouseY += (targetY - mouseY) * 0.05;

      ctx.clearRect(0, 0, width, height);

      // 1. Digital Frequency Sine Wave (Inspired by user sample 2)
      ctx.lineWidth = 1;
      const waveCount = 5;
      for (let w = 0; w < waveCount; w++) {
        ctx.beginPath();
        const offset = w * 18;
        const alpha = 0.15 - w * 0.025;
        ctx.strokeStyle = `rgba(245, 158, 11, ${Math.max(0.02, alpha)})`;

        const startY = height * 0.38 + offset;
        ctx.moveTo(0, startY);

        for (let x = 0; x < width; x += 8) {
          const distToMouse = Math.abs(x - mouseX);
          const mouseInfluence = Math.max(0, 1 - distToMouse / 400);
          const freq = 0.008 + w * 0.002;
          const amplitude = (25 + w * 12) * (1 + mouseInfluence * 1.5);
          const y = startY + Math.sin(x * freq + frame * 0.03 + w) * amplitude;
          ctx.lineTo(x, y);
        }
        ctx.stroke();
      }

      // 2. Interactive Coordinate Particles & Neural Links
      for (let i = 0; i < particleCount; i++) {
        const p = particles[i];
        p.x += p.vx;
        p.y += p.vy;

        if (p.x < 0) p.x = width;
        if (p.x > width) p.x = 0;
        if (p.y < 0) p.y = height;
        if (p.y > height) p.y = 0;

        // Draw particle
        ctx.fillStyle = `rgba(245, 158, 11, ${p.alpha})`;
        ctx.fillRect(p.x, p.y, p.size, p.size);

        // Distance to mouse
        const dx = mouseX - p.x;
        const dy = mouseY - p.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < 150) {
          ctx.beginPath();
          ctx.moveTo(p.x, p.y);
          ctx.lineTo(mouseX, mouseY);
          ctx.strokeStyle = `rgba(245, 158, 11, ${0.25 * (1 - dist / 150)})`;
          ctx.stroke();
        }

        // Connect nearby particles
        for (let j = i + 1; j < particleCount; j++) {
          const p2 = particles[j];
          const dxx = p.x - p2.x;
          const dyy = p.y - p2.y;
          const d = Math.sqrt(dxx * dxx + dyy * dyy);
          if (d < 90) {
            ctx.beginPath();
            ctx.moveTo(p.x, p.y);
            ctx.lineTo(p2.x, p2.y);
            ctx.strokeStyle = `rgba(255, 255, 255, ${0.08 * (1 - d / 90)})`;
            ctx.stroke();
          }
        }
      }

      // 3. Cybernetic Crosshairs at Cursor
      ctx.save();
      ctx.strokeStyle = 'rgba(245, 158, 11, 0.4)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(mouseX, mouseY, 28, 0, Math.PI * 2);
      ctx.stroke();

      // Crosshair tick marks
      const tick = 8;
      ctx.beginPath();
      ctx.moveTo(mouseX - 36, mouseY);
      ctx.lineTo(mouseX - 36 + tick, mouseY);
      ctx.moveTo(mouseX + 36, mouseY);
      ctx.lineTo(mouseX + 36 - tick, mouseY);
      ctx.moveTo(mouseX, mouseY - 36);
      ctx.lineTo(mouseX, mouseY - 36 + tick);
      ctx.moveTo(mouseX, mouseY + 36);
      ctx.lineTo(mouseX, mouseY + 36 - tick);
      ctx.stroke();

      // Technical coordinate readout text
      ctx.font = '10px "JetBrains Mono", monospace';
      ctx.fillStyle = 'rgba(245, 158, 11, 0.6)';
      ctx.fillText(`LOC // [${Math.round(mouseX)}, ${Math.round(mouseY)}]`, mouseX + 42, mouseY - 14);
      ctx.fillText(`ZERO-TRUST // ACTIVE`, mouseX + 42, mouseY + 2);
      ctx.restore();

      animationId = requestAnimationFrame(render);
    };

    render();

    return () => {
      window.removeEventListener('resize', handleResize);
      window.removeEventListener('mousemove', handleMouseMove);
      cancelAnimationFrame(animationId);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      className="fixed inset-0 pointer-events-none z-0 opacity-70"
    />
  );
};
