import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock any global objects or functions as needed
global.React = require('react');

window.HTMLElement.prototype.scrollIntoView = function() {}; 