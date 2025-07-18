# Privacy Impact Assessments

This directory contains Privacy Impact Assessments (PIAs) for features and changes that involve privacy-sensitive data or functionality.

## Purpose

Privacy Impact Assessments help identify and mitigate privacy risks before they occur. They are an essential part of our privacy-by-design approach and help ensure compliance with privacy regulations.

## When to Create a PIA

A Privacy Impact Assessment should be created when:

1. Implementing a new feature that collects, processes, or stores personal data
2. Modifying existing features that handle personal data
3. Changing data retention policies or practices
4. Implementing new data sharing mechanisms
5. Adding new analytics or tracking capabilities
6. Making architectural changes that affect data flow or storage

## How to Create a PIA

1. Copy the template from `docs/templates/privacy-impact-assessment.md`
2. Create a new file in this directory with a descriptive name (e.g., `user-profile-feature-pia.md`)
3. Fill out all sections of the template
4. Reference the PIA in your commit message using the format `PIA: <filename>` or `Privacy Impact Assessment: <filename>`

## Reviewing PIAs

PIAs should be reviewed by at least one other team member with knowledge of privacy requirements and best practices. The review should focus on:

1. Completeness of the assessment
2. Accuracy of risk evaluations
3. Effectiveness of proposed mitigation strategies
4. Compliance with relevant privacy regulations

## Automation

Our development workflow includes automated checks to ensure that changes to privacy-sensitive files have an associated PIA. The check can be bypassed in exceptional circumstances using the `--no-verify` flag with git commit, but this should be done only when absolutely necessary and with appropriate justification.