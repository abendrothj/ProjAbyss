#include "MarineCharacter.h"
#include "Camera/CameraComponent.h"
#include "Components/CapsuleComponent.h"
#include "GameFramework/CharacterMovementComponent.h"
#include "EnhancedInputComponent.h"
#include "EnhancedInputSubsystems.h"
#include "InteractableInterface.h"

AMarineCharacter::AMarineCharacter()
{
    PrimaryActorTick.bCanEverTick = true;

    // 1. Physics Tuning for Ships (CRITICAL)
    // By default, UE characters try to stay upright globally. 
    // We want to rotate with the ship so we don't slide off when it rocks.
    GetCharacterMovement()->bIgnoreBaseRotation = false; 
    
    // Ensure we keep the ship's momentum when we jump off (Newtonian physics)
    GetCharacterMovement()->bImpartBaseVelocityX = true;
    GetCharacterMovement()->bImpartBaseVelocityY = true;
    GetCharacterMovement()->bImpartBaseVelocityZ = true;

    // Walkable floor settings (Ships slope a lot in waves, so we increase the limit)
    GetCharacterMovement()->SetWalkableFloorAngle(60.f); 

    // 2. Camera Setup
    FirstPersonCameraComponent = CreateDefaultSubobject<UCameraComponent>(TEXT("FirstPersonCamera"));
    FirstPersonCameraComponent->SetupAttachment(GetCapsuleComponent());
    FirstPersonCameraComponent->SetRelativeLocation(FVector(-10.f, 0.f, 60.f)); // Standard Eye height
    FirstPersonCameraComponent->bUsePawnControlRotation = true;
}

void AMarineCharacter::BeginPlay()
{
    Super::BeginPlay();

    // Add Input Mapping Context
    if (APlayerController* PlayerController = Cast<APlayerController>(Controller))
    {
        if (UEnhancedInputLocalPlayerSubsystem* Subsystem = ULocalPlayer::GetSubsystem<UEnhancedInputLocalPlayerSubsystem>(PlayerController->GetLocalPlayer()))
        {
            if (DefaultMappingContext)
            {
                Subsystem->AddMappingContext(DefaultMappingContext, 0);
            }
        }
    }
}

void AMarineCharacter::SetupPlayerInputComponent(UInputComponent* PlayerInputComponent)
{
    Super::SetupPlayerInputComponent(PlayerInputComponent);

    if (UEnhancedInputComponent* EnhancedInputComponent = Cast<UEnhancedInputComponent>(PlayerInputComponent))
    {
        // Moving
        if (MoveAction)
        {
            EnhancedInputComponent->BindAction(MoveAction, ETriggerEvent::Triggered, this, &AMarineCharacter::Move);
        }

        // Looking
        if (LookAction)
        {
            EnhancedInputComponent->BindAction(LookAction, ETriggerEvent::Triggered, this, &AMarineCharacter::Look);
        }

        // Jumping
        if (JumpAction)
        {
            EnhancedInputComponent->BindAction(JumpAction, ETriggerEvent::Started, this, &ACharacter::Jump);
            EnhancedInputComponent->BindAction(JumpAction, ETriggerEvent::Completed, this, &ACharacter::StopJumping);
        }

        // Interaction (E key)
        if (InteractAction)
        {
            EnhancedInputComponent->BindAction(InteractAction, ETriggerEvent::Started, this, &AMarineCharacter::Interact);
        }
    }
}

void AMarineCharacter::Move(const FInputActionValue& Value)
{
    // We use FVector2D here because the Input Action is Axis2D (X, Y)
    FVector2D MovementVector = Value.Get<FVector2D>();

    if (Controller != nullptr)
    {
        // Y = Forward/Backward (W/S)
        AddMovementInput(GetActorForwardVector(), MovementVector.Y);
        // X = Right/Left (A/D)
        AddMovementInput(GetActorRightVector(), MovementVector.X);
    }
}

void AMarineCharacter::Look(const FInputActionValue& Value)
{
    // Mouse Input is also 2D (X, Y)
    FVector2D LookAxisVector = Value.Get<FVector2D>();

    if (Controller != nullptr)
    {
        // X = Yaw (Turning Left/Right)
        AddControllerYawInput(LookAxisVector.X);
        // Y = Pitch (Looking Up/Down)
        AddControllerPitchInput(LookAxisVector.Y);
    }
}

void AMarineCharacter::Interact()
{
    // Raycast forward to see if we are looking at the Ship Wheel or Loot
    AActor* HitActor = GetActorInView();
    if (HitActor)
    {
        // If the actor implements our interface, call Interact
        if (HitActor->GetClass()->ImplementsInterface(UInteractableInterface::StaticClass()))
        {
            IInteractableInterface::Execute_Interact(HitActor, this);
        }
        else
        {
            UE_LOG(LogTemp, Warning, TEXT("Actor %s does not implement InteractableInterface"), *HitActor->GetName());
        }
    }
}

AActor* AMarineCharacter::GetActorInView()
{
    if (!FirstPersonCameraComponent) return nullptr;

    FVector Start = FirstPersonCameraComponent->GetComponentLocation();
    FVector End = Start + (FirstPersonCameraComponent->GetForwardVector() * 300.f); // 3 Meters reach

    FHitResult HitResult;
    FCollisionQueryParams Params;
    Params.AddIgnoredActor(this);

    // Raycast against everything visible
    if (GetWorld()->LineTraceSingleByChannel(HitResult, Start, End, ECC_Visibility, Params))
    {
        return HitResult.GetActor();
    }
    return nullptr;
}

void AMarineCharacter::Tick(float DeltaTime)
{
    Super::Tick(DeltaTime);
}