#include "MasterShip.h"
#include "GameFramework/SpringArmComponent.h"
#include "Camera/CameraComponent.h"
#include "EnhancedInputComponent.h"
#include "EnhancedInputSubsystems.h"

AMasterShip::AMasterShip()
{
    PrimaryActorTick.bCanEverTick = true;

    // 1. Create the Hull Mesh
    HullMesh = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("HullMesh"));
    RootComponent = HullMesh;
    
    // Physics Setup
    HullMesh->SetSimulatePhysics(true);
    HullMesh->SetMassOverrideInKg(NAME_None, 1000.0f, true);
    HullMesh->SetLinearDamping(1.0f);  // Air resistance/Water drag base
    HullMesh->SetAngularDamping(2.0f); // Rotational resistance

    // 2. Camera Setup (So you can see the boat!)
    CameraBoom = CreateDefaultSubobject<USpringArmComponent>(TEXT("CameraBoom"));
    CameraBoom->SetupAttachment(RootComponent);
    CameraBoom->TargetArmLength = 800.0f;
    CameraBoom->SetRelativeRotation(FRotator(-30.f, 0.f, 0.f));
    CameraBoom->bEnableCameraLag = true; // Smooths out the wave jitter
    CameraBoom->CameraLagSpeed = 3.0f;

    FollowCamera = CreateDefaultSubobject<UCameraComponent>(TEXT("FollowCamera"));
    FollowCamera->SetupAttachment(CameraBoom, USpringArmComponent::SocketName);

    // 3. Pontoons
    // We place them in a square around the center.
    PontoonFL = CreateDefaultSubobject<USceneComponent>(TEXT("PontoonFL"));
    PontoonFL->SetupAttachment(RootComponent);
    PontoonFL->SetRelativeLocation(FVector(200.f, -150.f, -50.f));

    PontoonFR = CreateDefaultSubobject<USceneComponent>(TEXT("PontoonFR"));
    PontoonFR->SetupAttachment(RootComponent);
    PontoonFR->SetRelativeLocation(FVector(200.f, 150.f, -50.f));

    PontoonBL = CreateDefaultSubobject<USceneComponent>(TEXT("PontoonBL"));
    PontoonBL->SetupAttachment(RootComponent);
    PontoonBL->SetRelativeLocation(FVector(-200.f, -150.f, -50.f));

    PontoonBR = CreateDefaultSubobject<USceneComponent>(TEXT("PontoonBR"));
    PontoonBR->SetupAttachment(RootComponent);
    PontoonBR->SetRelativeLocation(FVector(-200.f, 150.f, -50.f));
}

void AMasterShip::BeginPlay()
{
    Super::BeginPlay();

    // -- ENHANCED INPUT INITIALIZATION --
    // This adds the "Keyboard Map" (Context) to the local player so pressing W actually does something.
    if (APlayerController* PlayerController = Cast<APlayerController>(Controller))
    {
        ApplyInputMappingToController(PlayerController);
    }
}

void AMasterShip::PossessedBy(AController* NewController)
{
    Super::PossessedBy(NewController);
    if (APlayerController* PC = Cast<APlayerController>(NewController))
    {
        ApplyInputMappingToController(PC);
    }
}

// -- INPUT BINDING --
void AMasterShip::SetupPlayerInputComponent(UInputComponent* PlayerInputComponent)
{
    Super::SetupPlayerInputComponent(PlayerInputComponent);

    // Cast to Enhanced Input Component
    if (UEnhancedInputComponent* EnhancedInputComponent = Cast<UEnhancedInputComponent>(PlayerInputComponent))
    {
        // Bind the "Move" Action (W/S)
        if (MoveAction)
        {
            // Triggered: Runs every frame the key is held
            EnhancedInputComponent->BindAction(MoveAction, ETriggerEvent::Triggered, this, &AMasterShip::MoveForward);
            // Completed: Runs once when key is released (resets speed to 0)
            EnhancedInputComponent->BindAction(MoveAction, ETriggerEvent::Completed, this, &AMasterShip::MoveForward);
        }

        // Bind the "Turn" Action (A/D)
        if (TurnAction)
        {
            EnhancedInputComponent->BindAction(TurnAction, ETriggerEvent::Triggered, this, &AMasterShip::TurnRight);
            EnhancedInputComponent->BindAction(TurnAction, ETriggerEvent::Completed, this, &AMasterShip::TurnRight);
        }

        // Allow exiting the ship with the same Interact key
        if (InteractAction)
        {
            EnhancedInputComponent->BindAction(InteractAction, ETriggerEvent::Started, this, &AMasterShip::HandleExitInput);
        }
    }
}

void AMasterShip::MoveForward(const FInputActionValue& Value)
{
    // Enhanced Input gives us a float (-1.0 to 1.0)
    CurrentThrottle = Value.Get<float>();
}

void AMasterShip::TurnRight(const FInputActionValue& Value)
{
    // Enhanced Input gives us a float (-1.0 to 1.0)
    CurrentSteering = Value.Get<float>();
}

// -- PHYSICS LOOP --
void AMasterShip::Tick(float DeltaTime)
{
    Super::Tick(DeltaTime);

    // 1. Update Ocean Time (So waves move)
    OceanSolver.Time = GetWorld()->GetTimeSeconds();

    // 2. Buoyancy Logic
    USceneComponent* Pontoons[] = { PontoonFL, PontoonFR, PontoonBL, PontoonBR };
    int32 PontoonsUnderwater = 0; // Track how many are touching water

    for (USceneComponent* Pontoon : Pontoons)
    {
        FVector PontoonLoc = Pontoon->GetComponentLocation();
        
        // ASK THE MATH: "How high is the wave here?"
        float WaveHeight = OceanSolver.GetWaveHeightAt(PontoonLoc);

        // CHECK: Is the pontoon below that wave?
        if (PontoonLoc.Z < WaveHeight)
        {
            PontoonsUnderwater++;
            float Depth = WaveHeight - PontoonLoc.Z;

            // PHYSICS: Apply Upward Force
            FVector UpwardForce = FVector(0, 0, Depth * FloatForce);
            HullMesh->AddForceAtLocation(UpwardForce, PontoonLoc);

            // DRAG: Apply Water Resistance (stops it from bouncing forever)
            FVector VelocityAtPoint = HullMesh->GetPhysicsLinearVelocityAtPoint(PontoonLoc);
            HullMesh->AddForceAtLocation(-VelocityAtPoint * WaterDrag, PontoonLoc);
        }
    }

    // 3. MOVEMENT LOGIC
    // We only allow the engine to work if at least one pontoon is in the water.
    // This prevents "Flying Ship" bugs if you launch off a huge wave.
    if (PontoonsUnderwater > 0)
    {
        // Forward Engine
        if (CurrentThrottle != 0.f)
        {
            // Calculate force direction based on where the boat is facing
            FVector ForwardDir = GetActorForwardVector();
            // Force = Direction * Power * Input (-1 or 1)
            FVector EngineForce = ForwardDir * EnginePower * CurrentThrottle;
            
            // Apply force at center of mass
            HullMesh->AddForce(EngineForce);
        }

        // Steering Rudder
        if (CurrentSteering != 0.f)
        {
            // Torque = Up Vector (Z) * Power * Input
            // We use AddTorqueInDegrees because it's easier to tune than raw Torque
            FVector Torque = FVector(0, 0, 1) * TurnSpeed * CurrentSteering;
            HullMesh->AddTorqueInDegrees(Torque);
        }
    }
}

void AMasterShip::ApplyInputMappingToController(AController* InController)
{
    if (!InController) return;
    if (APlayerController* PlayerController = Cast<APlayerController>(InController))
    {
        if (UEnhancedInputLocalPlayerSubsystem* Subsystem = ULocalPlayer::GetSubsystem<UEnhancedInputLocalPlayerSubsystem>(PlayerController->GetLocalPlayer()))
        {
            if (DefaultMappingContext)
            {
                Subsystem->AddMappingContext(DefaultMappingContext, 1); // priority above character
            }
        }

        // Snap camera/view to ship's follow camera
        if (FollowCamera)
        {
            PlayerController->SetViewTarget(this);
        }
    }
}

void AMasterShip::RemoveShipInputMappingFromController(AController* InController)
{
    if (!InController) return;
    if (APlayerController* PlayerController = Cast<APlayerController>(InController))
    {
        if (UEnhancedInputLocalPlayerSubsystem* Subsystem = ULocalPlayer::GetSubsystem<UEnhancedInputLocalPlayerSubsystem>(PlayerController->GetLocalPlayer()))
        {
            if (DefaultMappingContext)
            {
                Subsystem->RemoveMappingContext(DefaultMappingContext);
            }
        }
    }
}

void AMasterShip::Interact_Implementation(APawn* InstigatorPawn)
{
    if (!InstigatorPawn) return;

    // Get the instigator's controller
    AController* InstigatorController = InstigatorPawn->GetController();
    if (!InstigatorController)
    {
        return;
    }

    // Cache the pawn so we can return later
    LastDriverPawn = InstigatorPawn;

    // Unpossess the instigator pawn
    InstigatorController->UnPossess();

    // Possess this ship
    InstigatorController->Possess(this);

    // Apply input mapping and snap view
    ApplyInputMappingToController(InstigatorController);

    // Optional: disable input on the instigator pawn to avoid conflicts
    InstigatorPawn->DisableInput(Cast<APlayerController>(InstigatorController));
}

void AMasterShip::ExitToCachedPawn(AController* InController)
{
    if (!InController) return;
    APawn* CachedPawn = LastDriverPawn.Get();
    if (!CachedPawn) return;

    InController->UnPossess();
    InController->Possess(CachedPawn);

    RemoveShipInputMappingFromController(InController);

    if (APlayerController* PC = Cast<APlayerController>(InController))
    {
        if (CachedPawn->GetController() == InController)
        {
            PC->SetViewTarget(CachedPawn);
        }
    }

    CachedPawn->EnableInput(Cast<APlayerController>(InController));
    LastDriverPawn.Reset();
}

void AMasterShip::HandleExitInput(const FInputActionValue& Value)
{
    if (Value.Get<bool>() == false)
    {
        return; // only on press
    }

    if (AController* C = GetController())
    {
        ExitToCachedPawn(C);
    }
}
