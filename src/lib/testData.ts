// src/lib/testData.ts

export const testCollections = [
    {
      name: "Web Development Basics",
      studyMaterials: [
        { type: "markdown" as const, content: "# HTML Basics\n\nHTML (HyperText Markup Language) is the standard markup language for creating web pages.\n\n## Key Concepts\n\n- Tags\n- Elements\n- Attributes" }
      ],
      reviewMaterials: [
        { question: "What does HTML stand for?", answer: "HyperText Markup Language" },
        { question: "What is the purpose of CSS?", answer: "CSS (Cascading Style Sheets) is used to style and layout web pages" }
      ]
    },
    {
      name: "JavaScript Fundamentals",
      studyMaterials: [
        { type: "markdown" as const, content: "# JavaScript Variables\n\nIn JavaScript, you can declare variables using `var`, `let`, or `const`.\n\n```javascript\nlet x = 5;\nconst y = 10;\n```" }
      ],
      reviewMaterials: [
        { question: "What is the difference between let and const?", answer: "let allows reassignment, while const creates a read-only reference" },
        { question: "What is a closure in JavaScript?", answer: "A closure is a function that has access to variables in its outer (enclosing) lexical scope" }
      ]
    }
  ];