# S00 Windows CPU owned ASCII input boundary

- 상태: completed
- 담당: guardian_fault
- 우선순위: P0
- 요구사항: R01, R08
- 파일: 독점 scripts/windows_cpu_probe.ps1, 전용 workitem/validation/evidence
- 의존: frozen38e7 WindowsCPU, 기존worker미시작typed/after-start 실패
- 가설: 고정VK_A+잘못된scan0 CHAR조합은 실제키와 불일치; winit modifiers는 keyboard cache, egui command+A는selectall. CHAR-only는winit pendingevent없을때 무시할수있음.
- 인수: user keyboardstate/SendInput/foreground/clipboard변경 없이 ownedHWND boundedASCII CHAR-only 대조1회; 실제PNG/worker/cleanup 기록. 실패로 제품StopExit불합격 단정금지, 실제worker시작/stop성공 전완료 주장금지. 새가설만추가실험.

문자별VkKeyScanW lowVK+MapVirtualKeyW scan 대조를 root 승인했다. shift요구highbyte는읽기 mapping 결과로만 사용하고 modifier조작하지 않음. 원문 WM_CHAR 유지, 사용자 keyboardstate 불변. 29 ASCII chars 유한1trial. CHAR-only 실제실패 typed빈칸은 예상ignored경로와 일치.

문자별VK+scan대조 실제1회 exit0/worker18312 시작/StopExit0/소유cleanup/desktop불변. CHAR-only 실패보존. 새default는matchedkeys true, legacy시험은 -MatchedKeys:$false로명시가능. 제품수정없음. 정확modifier기전은미확정.
