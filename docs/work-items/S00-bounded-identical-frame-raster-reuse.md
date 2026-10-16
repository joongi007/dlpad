# S00: 동일 프레임의 bounded raster 재사용 실험

- 상태: in-progress
- 분류: performance
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: root crates/app/src/software_host.rs 통합·실제검증·문서; soak_recovery_audit /tmp/dlpad-frame-reuse.rs 초안만
- 선행 작업: S00-gui-idle-window-attribution; 실제2ms 완료/Next/Stop fixture 결과 확보

## 사용자 결과

내용이 동일한 프레임에서 불필요한 전체 software raster 비용을 줄이는 실험이다. 아직 제품에 채택하거나 성능 개선을 검증하지 않았다.

## 계약과 범위

Rust native 기존 renderer/direct path 유지, dependency/Cargo/schema/thread 추가0. egui UI/platform/accessibility 처리와 전체 present는 계속한다. 동일한 원mesh/clip/scale/size를 정확 비교하고 texture set/free가 없으며 원surface age1인 경우만 raster 재사용 후보로 한다. 이전primitive 보관은checked2MiB 상한, 초과/overflow/불확실성은 full render. 추가pixel canvas0. hidden/zero-size/resize/scale/texture/failedpresent에서무효화한다. 아직root만제품수정가능하며native sourceguard 측정도중 제품수정금지다.

## 인수 조건

변경vertex/color/clip/texture set/free·hidden·size/scale·age0/2·메모리초과 실패회귀. 실제private GUI Next/theme/scroll/resize/숨김복귀 screenshot과원3x60s strict10초 CPU1%/memory80MiB를검증한다. 기존원FAIL 보존하며 hash-only 시각동일 판정 금지.

## 검증과 handoff

로컬dependency API read-only 검토 초안이있으며compile/native/제품변경미수행이다. root가현재2ms 기능인수를끝낸뒤실험채택여부를판단한다. main CPU 관측은 raster 원인확정이아니다.

하위초안/tmp/dlpad-frame-reuse.rs(SHA69f674b1c8a3b9badce41b040d26baf54c1a1f493a99464d1d91867c304e6902)를확보했다. Mesh/clip 정확동등·Vec 실제capacity checked2MiB·성공present후Vec move·age0/2 및texture delta무효화의순수3회귀를제안한다. 아직compile/제품변경은없으며현재2ms Stop native sourceguard가끝난뒤root가최소통합한다. 이전화면벡터의보관비용은최대2MiB이며추가pixel canvas/dependency/thread0. 기존전체MAX_VERTICES/INDEX상한과독립된더작은보관상한이다.

root가software_host.rs 한파일에helper를통합했다. 첫draw/hidden/zero-size/resize/texturedelta/age0,2와present실패는full render 또는cache무효화로되돌린다. egui업데이트/tessellation/validation/platform/accessibility와전체present는계속하고동일Mesh/clip/scale/size일때fill/raster/XRGBmask만생략한다. Vec clone없고성공present후ownership move, retained2MiB에는Frame struct와실제Vec capacity를checked합산했다. Nativeformatter는exit0/격리inputunchanged이며기대한소스변경으로sourceguard는고정검증과구분한다. 현재Windowswhole4실행중이며GUI 비용·화면변경인수전이다.

Windowswhole4와root source543/原Job自然0/input/private/현재PE대조PASS, cache순수3회귀의실제Windows실행도확인했다. 현재PE ac0ad5abb7f28c3da59fce391d3189afea409f485a678ec7c062a79718a612e6/7917056B, 앞선71ec default대비2048B증가다. 원pixel캔버스/依存/thread 추가0, retained최대2MiB와정확mesh비교비용이추가된다. 下一GUI3x60/원strict10s·80MiB 및실제UI pixel변경/숨김복귀를인수하기전성능개선완료로표현하지않는다. soak_recovery_audit는현재파일에대한/tmp통합안전감사를read-only로진행한다. 원GUIfixture3만payload 검증은별도root I/O로끝낸뒤새CPU측정을시작한다.

ac0ad5 GUI3회는avg0.676907/0.468057/0.442421%, max10초1.252634/0.780823/0.772848%, memory3PASS로전체CPU FAIL이다. 첫round3창에library781250/restore156250/metrics156250/무명156250tick가관측됐고thread churn의불완전한귀속을보존한다. root원18창수식/source543/compiledpins/Jobguards대조 및全3만payload/3DB각1만행/CAP/integrity 보존도완료했다. 원FAIL을유지하고S01-library-idle-wait-and-stop-wake 후속실험으로quiet비용을좁힌다. Rendererpixel인수는Next/theme/200%기존helper와새Resize standalone/tmp초안을조정중이며현재CPUac0결과는quiet500변경전소스증거다.
