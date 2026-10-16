# CPU 후보의 펼침 상태 접근성 통합

Linux CPU snapshot `608eeeb5a2f3a046e0a389ca788770f58e6ff38369d056ef8c8debdb262829c9` /12,403,064B에서 실행 계획 헤더의 AT-SPI Expandable/Expanded를 실제 GUI로 확인했다. [원시 결과](evidence/cpu-expanded-state/plan-native/result.json)는 마우스·native action·Enter·Space closed/open/closed 모두 실제 미리보기 존재와 상태 bit가 일치한다. 정상 종료0·소유 정리·시작/종료binarySHA 일치를 확인했다. 사용자 desktop/clipboard·실제 영상·외부 도구 실행은 사용하지 않았다.

root의 UI helper는 egui CollapsingState의 목표 open 상태를 공개 AccessKit node builder에 전달한다. 애니메이션의 body 존재/openness 값으로 상태를 추정하지 않는다. Linux vendor bridge는 명시된 expanded 값이 있을 때 Expandable, true일 때 Expanded를 전달한다. 실제 adapter Tree 업데이트의 StateChanged 이벤트 회귀도 포함했다. 새 dependency/schema/Cargo 변경은 없고 기존12,402,040B snapshot 대비+1,024B다. 여러 시험·코드 변경이 함께 포함돼 차이를 단일 함수 비용으로 귀속하지 않는다.

누락의 [실패 재현 요약](evidence/cpu-expanded-state/red-reproduction.txt)과 수정 중 발견한 [컴파일 실패](evidence/cpu-expanded-state/adapter-green.log)를 보존했다. consumer API는 `data().is_expanded()`이며 수정한 [실제 adapter 회귀](evidence/cpu-expanded-state/adapter-green-retry.log)가 통과했다.

[Linux 전체 실행 결과](evidence/cpu-expanded-state/workspace-result.json): fmt, 기본clippy, 기본test200PASS/5ignored, CPUclippy, CPUtest339PASS/5ignored, 기본workspace release, CPUrelease 모두exit0. [Windows GNU native](evidence/cpu-expanded-state/windows/result.json)는 clippy/test255PASS/1ignored/release와 입력258개 SHA before/after/current 동일, input desktop unchanged를 확인했다. Windows 바이너리7,246,336B/SHA `a6caa91a28ff0c6f5002902adb388ad5893f2071902342319bdfdd127a8adbad`다. GUI/IME/clipboard/MSVC/macOS 검증으로 확대하지 않는다.

[진단·MPD 실제 GUI](cpu-diagnostic-expander-validation.md)도 두 헤더의 closed/open/closed/reopen에서 Expandable/Expanded와 원 Failed job/Attempt/옵션·MPD bytes 보존을 확인했다. root는 [DB·상태·소유 PID](evidence/cpu-diagnostic-expander/current-1/root-review.json)를 독립 검증하고 mpd-3.png의 펼침과 BaseUrl/Initialization 본문을 확인했다. 전체 음성 reader·다른 OS·모든 배율·최종 스택과 P0 완료는 아직 미검증이다. [통합 작업](../work-items/S00-collapsing-accessibility-integration.md)을 따른다.
